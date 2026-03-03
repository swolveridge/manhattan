use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use petgraph::graph::DiGraph;
use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::Value;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub strict: bool,
    pub warnings_as_errors: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub specs: Vec<ParsedSpec>,
    pub graph: SpecGraph,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSpec {
    pub file_name: String,
    pub kind: Option<SpecKind>,
    pub specifies: Vec<SpecTarget>,
    pub headings: Vec<Heading>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecKind {
    Feature,
    Behavioural,
    Interface,
    Constraint,
    Context,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecTarget {
    pub raw: String,
    pub file_name: String,
    pub heading_slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub slug: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    pub kind: GraphNodeKind,
    pub file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphNodeKind {
    File,
    Section,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub kind: GraphEdgeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphEdgeKind {
    Specifies,
    Contains,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    IoReadFailure,
    InvalidFrontMatter,
    InvalidKind,
    InvalidSpecifies,
    InvalidSpecifiesTarget,
    NestedSpecFile,
    InvalidCrossReference,
    BrokenTargetFile,
    BrokenTargetHeading,
    BrokenCrossReferenceFile,
    BrokenCrossReferenceHeading,
    CycleDetected,
    OrphanSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file_name: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("failed to list specs directory {path}: {source}")]
    ReadDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Deserialize)]
struct RawFrontMatter {
    #[serde(rename = "Kind")]
    kind: Option<String>,
    #[serde(rename = "Specifies")]
    specifies: Option<Value>,
}

struct RawSpecFile {
    file_name: String,
    headings: Vec<Heading>,
    specifies: Vec<SpecTarget>,
    kind: Option<SpecKind>,
}

const SPEC_FILE_EXEMPTION: &str = "spec-format.md";

pub fn parse_specs_directory(
    specs_dir: impl AsRef<Path>,
    options: ParseOptions,
) -> Result<ParseResult, ParserError> {
    let specs_dir = specs_dir.as_ref();
    if !specs_dir.is_dir() {
        return Err(ParserError::ReadDirectory {
            path: specs_dir.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "directory not found"),
        });
    }

    let mut diagnostics = Vec::new();
    let mut top_level_markdown_paths = Vec::new();

    for entry in WalkDir::new(specs_dir).min_depth(1) {
        let entry = match entry {
            Ok(value) => value,
            Err(err) => {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode::IoReadFailure,
                    message: format!("Read directory entry failed: {err}"),
                    location: None,
                });
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let is_markdown = entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if !is_markdown {
            continue;
        }

        if entry.depth() > 1 {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::NestedSpecFile,
                message: format!(
                    "Reject nested spec file '{}' because specs/ must stay flat.",
                    path_to_display_name(specs_dir, entry.path())
                ),
                location: Some(SourceLocation {
                    file_name: path_to_display_name(specs_dir, entry.path()),
                    line: None,
                    column: None,
                }),
            });
            continue;
        }

        top_level_markdown_paths.push(entry.path().to_path_buf());
    }

    top_level_markdown_paths.sort();
    let discovered_file_names: BTreeSet<String> = top_level_markdown_paths
        .iter()
        .map(|p| path_to_display_name(specs_dir, p))
        .collect();

    let mut raw_specs = Vec::new();
    for path in &top_level_markdown_paths {
        raw_specs.push(parse_single_file(
            specs_dir,
            path,
            &mut diagnostics,
            &discovered_file_names,
        ));
    }

    let headings_by_file: HashMap<String, HashSet<String>> = raw_specs
        .iter()
        .map(|spec| {
            let slugs = spec
                .headings
                .iter()
                .map(|h| h.slug.clone())
                .collect::<HashSet<_>>();
            (spec.file_name.clone(), slugs)
        })
        .collect();

    let mut graph_nodes = Vec::new();
    let mut node_ids = HashSet::new();
    for spec in &raw_specs {
        insert_node(
            &mut graph_nodes,
            &mut node_ids,
            GraphNode {
                id: spec.file_name.clone(),
                kind: GraphNodeKind::File,
                file_name: spec.file_name.clone(),
            },
        );
        for heading in &spec.headings {
            insert_node(
                &mut graph_nodes,
                &mut node_ids,
                GraphNode {
                    id: format!("{}#{}", spec.file_name, heading.slug),
                    kind: GraphNodeKind::Section,
                    file_name: spec.file_name.clone(),
                },
            );
        }
    }

    let mut graph_edges = Vec::new();
    let mut valid_outgoing_specifies_count: HashMap<String, usize> = HashMap::new();

    for spec in &raw_specs {
        for heading in &spec.headings {
            graph_edges.push(GraphEdge {
                source: format!("{}#{}", spec.file_name, heading.slug),
                target: spec.file_name.clone(),
                kind: GraphEdgeKind::Contains,
            });
        }
    }

    for spec in &raw_specs {
        for target in &spec.specifies {
            if !discovered_file_names.contains(&target.file_name) {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode::BrokenTargetFile,
                    message: format!(
                        "Resolve Specifies target '{}' in '{}' to an existing file.",
                        target.raw, spec.file_name
                    ),
                    location: Some(SourceLocation {
                        file_name: spec.file_name.clone(),
                        line: None,
                        column: None,
                    }),
                });
                continue;
            }

            let exists = headings_by_file
                .get(&target.file_name)
                .is_some_and(|h| h.contains(&target.heading_slug));
            if !exists {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode::BrokenTargetHeading,
                    message: format!(
                        "Resolve Specifies target '{}' in '{}' to an existing heading.",
                        target.raw, spec.file_name
                    ),
                    location: Some(SourceLocation {
                        file_name: spec.file_name.clone(),
                        line: None,
                        column: None,
                    }),
                });
                continue;
            }

            graph_edges.push(GraphEdge {
                source: spec.file_name.clone(),
                target: format!("{}#{}", target.file_name, target.heading_slug),
                kind: GraphEdgeKind::Specifies,
            });
            *valid_outgoing_specifies_count
                .entry(spec.file_name.clone())
                .or_insert(0) += 1;
        }
    }

    let cycles = find_cycles(&graph_nodes, &graph_edges);
    for cycle in cycles {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::CycleDetected,
            message: format!("Break directed cycle: {}", cycle.join(" -> ")),
            location: None,
        });
    }

    for spec in &raw_specs {
        if spec.file_name == SPEC_FILE_EXEMPTION {
            continue;
        }
        if matches!(
            spec.kind,
            Some(SpecKind::Feature | SpecKind::Behavioural | SpecKind::Interface)
        ) && valid_outgoing_specifies_count
            .get(&spec.file_name)
            .copied()
            .unwrap_or(0)
            == 0
        {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Warning,
                code: DiagnosticCode::OrphanSpec,
                message: format!(
                    "Add at least one valid Specifies target for '{}', or change Kind if it should float.",
                    spec.file_name
                ),
                location: Some(SourceLocation {
                    file_name: spec.file_name.clone(),
                    line: Some(1),
                    column: Some(1),
                }),
            });
        }
    }

    apply_warning_policy(&mut diagnostics, options);

    let parsed_specs = raw_specs
        .into_iter()
        .map(|raw| ParsedSpec {
            file_name: raw.file_name,
            kind: raw.kind,
            specifies: raw.specifies,
            headings: raw.headings,
        })
        .collect::<Vec<_>>();

    Ok(ParseResult {
        specs: parsed_specs,
        graph: SpecGraph {
            nodes: graph_nodes,
            edges: graph_edges,
        },
        diagnostics,
    })
}

fn parse_single_file(
    specs_dir: &Path,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
    known_files: &BTreeSet<String>,
) -> RawSpecFile {
    let file_name = path_to_display_name(specs_dir, path);
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::IoReadFailure,
                message: format!("Read '{}' failed: {err}", file_name),
                location: Some(SourceLocation {
                    file_name: file_name.clone(),
                    line: None,
                    column: None,
                }),
            });
            return RawSpecFile {
                file_name,
                headings: Vec::new(),
                specifies: Vec::new(),
                kind: None,
            };
        }
    };

    let (front_matter, body, fm_lines) = parse_front_matter(&file_name, &content, diagnostics);
    let headings = extract_headings(&body);
    validate_cross_references(&file_name, &body, &headings, known_files, diagnostics);

    let mut kind = None;
    let mut specifies = Vec::new();
    if let Some(raw_fm) = &front_matter {
        kind = parse_kind(&file_name, raw_fm.kind.as_deref(), diagnostics);
        specifies = parse_specifies(&file_name, raw_fm.specifies.as_ref(), fm_lines, diagnostics);
    }

    RawSpecFile {
        file_name,
        headings,
        specifies,
        kind,
    }
}

fn parse_front_matter(
    file_name: &str,
    content: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Option<RawFrontMatter>, String, usize) {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::InvalidFrontMatter,
            message: "Start file with YAML front matter delimited by '---'.".to_string(),
            location: Some(SourceLocation {
                file_name: file_name.to_string(),
                line: Some(1),
                column: Some(1),
            }),
        });
        return (None, content.to_string(), 0);
    }

    let mut yaml_lines = Vec::new();
    let mut front_matter_end_line = 0usize;
    for (index, line) in content.lines().enumerate().skip(1) {
        if line == "---" {
            front_matter_end_line = index + 1;
            break;
        }
        yaml_lines.push(line);
    }

    if front_matter_end_line == 0 {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::InvalidFrontMatter,
            message: "Close YAML front matter with a terminating '---' line.".to_string(),
            location: Some(SourceLocation {
                file_name: file_name.to_string(),
                line: Some(1),
                column: Some(1),
            }),
        });
        return (None, content.to_string(), 0);
    }

    let yaml_block = yaml_lines.join("\n");
    let parsed: RawFrontMatter = match serde_yaml::from_str(&yaml_block) {
        Ok(value) => value,
        Err(err) => {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::InvalidFrontMatter,
                message: format!(
                    "Parse YAML front matter in '{}' correctly: {err}",
                    file_name
                ),
                location: Some(SourceLocation {
                    file_name: file_name.to_string(),
                    line: Some(1),
                    column: Some(1),
                }),
            });
            RawFrontMatter {
                kind: None,
                specifies: None,
            }
        }
    };

    let body = content
        .lines()
        .skip(front_matter_end_line)
        .collect::<Vec<_>>()
        .join("\n");
    (Some(parsed), body, front_matter_end_line)
}

fn parse_kind(
    file_name: &str,
    raw_kind: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SpecKind> {
    let Some(kind) = raw_kind else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::InvalidKind,
            message: format!("Declare Kind in front matter for '{}'.", file_name),
            location: Some(SourceLocation {
                file_name: file_name.to_string(),
                line: Some(1),
                column: Some(1),
            }),
        });
        return None;
    };

    match kind {
        "feature" => Some(SpecKind::Feature),
        "behavioural" => Some(SpecKind::Behavioural),
        "interface" => Some(SpecKind::Interface),
        "constraint" => Some(SpecKind::Constraint),
        "context" => Some(SpecKind::Context),
        _ => {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::InvalidKind,
                message: format!(
                    "Use a valid Kind in '{}': feature | behavioural | interface | constraint | context.",
                    file_name
                ),
                location: Some(SourceLocation {
                    file_name: file_name.to_string(),
                    line: Some(1),
                    column: Some(1),
                }),
            });
            None
        }
    }
}

fn parse_specifies(
    file_name: &str,
    raw_specifies: Option<&Value>,
    _front_matter_end_line: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<SpecTarget> {
    let Some(raw_specifies) = raw_specifies else {
        return Vec::new();
    };

    let Some(entries) = raw_specifies.as_sequence() else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::InvalidSpecifies,
            message: format!(
                "Set Specifies in '{}' as a YAML list of target strings.",
                file_name
            ),
            location: Some(SourceLocation {
                file_name: file_name.to_string(),
                line: Some(1),
                column: Some(1),
            }),
        });
        return Vec::new();
    };

    let target_regex = Regex::new(r"^([A-Za-z0-9._-]+\.md)#(.+)$").expect("regex must compile");
    let mut targets = Vec::new();
    for entry in entries {
        let Some(raw) = entry.as_str() else {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::InvalidSpecifies,
                message: format!(
                    "Use string entries in Specifies for '{}'; reject non-string values.",
                    file_name
                ),
                location: Some(SourceLocation {
                    file_name: file_name.to_string(),
                    line: Some(1),
                    column: Some(1),
                }),
            });
            continue;
        };

        let Some(captures) = target_regex.captures(raw) else {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::InvalidSpecifiesTarget,
                message: format!(
                    "Use Specifies target format file.md#heading-slug in '{}': '{raw}'.",
                    file_name
                ),
                location: Some(SourceLocation {
                    file_name: file_name.to_string(),
                    line: Some(1),
                    column: Some(1),
                }),
            });
            continue;
        };

        let target_file = captures[1].to_string();
        let target_heading_raw = captures[2].trim();
        if target_heading_raw.is_empty() {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::InvalidSpecifiesTarget,
                message: format!(
                    "Use non-empty heading slug in Specifies target for '{}': '{raw}'.",
                    file_name
                ),
                location: Some(SourceLocation {
                    file_name: file_name.to_string(),
                    line: Some(1),
                    column: Some(1),
                }),
            });
            continue;
        }

        targets.push(SpecTarget {
            raw: raw.to_string(),
            file_name: target_file,
            heading_slug: normalise_slug(target_heading_raw),
        });
    }

    targets
}

fn extract_headings(body: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut counts = HashMap::<String, usize>::new();

    for (index, raw_line) in body.lines().enumerate() {
        let line = raw_line.trim_start();
        if !line.starts_with('#') {
            continue;
        }

        let level = line.chars().take_while(|c| *c == '#').count();
        if level == 0 || level > 6 {
            continue;
        }

        let after_hashes = &line[level..];
        if !after_hashes.starts_with(' ') {
            continue;
        }

        let text = after_hashes.trim().to_string();
        if text.is_empty() {
            continue;
        }

        let base_slug = normalise_slug(&text);
        let slug = if let Some(existing_count) = counts.get_mut(&base_slug) {
            *existing_count += 1;
            format!("{}-{}", base_slug, *existing_count)
        } else {
            counts.insert(base_slug.clone(), 0);
            base_slug
        };

        headings.push(Heading {
            level: level as u8,
            text,
            slug,
            line: index + 1,
        });
    }
    headings
}

fn validate_cross_references(
    file_name: &str,
    body: &str,
    headings: &[Heading],
    known_files: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let known_local_headings = headings
        .iter()
        .map(|h| h.slug.clone())
        .collect::<HashSet<_>>();
    let md_target = Regex::new(r"^([A-Za-z0-9._-]+\.md)(?:#(.+))?$").expect("regex must compile");

    let parser = Parser::new_ext(body, Options::all());
    for event in parser {
        let Event::Start(Tag::Link { dest_url, .. }) = event else {
            continue;
        };

        let destination = dest_url.to_string();
        if destination.starts_with("http://")
            || destination.starts_with("https://")
            || destination.starts_with("mailto:")
        {
            continue;
        }

        if let Some(local_slug_raw) = destination.strip_prefix('#') {
            let local_slug = normalise_slug(local_slug_raw);
            if !known_local_headings.contains(&local_slug) {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Warning,
                    code: DiagnosticCode::BrokenCrossReferenceHeading,
                    message: format!(
                        "Resolve local heading link '#{}' in '{}'.",
                        local_slug_raw, file_name
                    ),
                    location: Some(SourceLocation {
                        file_name: file_name.to_string(),
                        line: None,
                        column: None,
                    }),
                });
            }
            continue;
        }

        let Some(captures) = md_target.captures(&destination) else {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Warning,
                code: DiagnosticCode::InvalidCrossReference,
                message: format!(
                    "Use local spec link format file.md or file.md#heading-slug in '{}': '{}'.",
                    file_name, destination
                ),
                location: Some(SourceLocation {
                    file_name: file_name.to_string(),
                    line: None,
                    column: None,
                }),
            });
            continue;
        };

        let target_file = captures[1].to_string();
        if !known_files.contains(&target_file) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Warning,
                code: DiagnosticCode::BrokenCrossReferenceFile,
                message: format!(
                    "Resolve cross-reference '{}' in '{}' to an existing spec file.",
                    destination, file_name
                ),
                location: Some(SourceLocation {
                    file_name: file_name.to_string(),
                    line: None,
                    column: None,
                }),
            });
        }
    }
}

fn find_cycles(nodes: &[GraphNode], edges: &[GraphEdge]) -> Vec<Vec<String>> {
    let mut index_by_id = BTreeMap::<String, usize>::new();
    for (idx, node) in nodes.iter().enumerate() {
        index_by_id.insert(node.id.clone(), idx);
    }

    let mut graph = DiGraph::<String, ()>::new();
    let mut node_indices = Vec::new();
    for node in nodes {
        node_indices.push(graph.add_node(node.id.clone()));
    }

    for edge in edges {
        let Some(source_idx) = index_by_id.get(&edge.source) else {
            continue;
        };
        let Some(target_idx) = index_by_id.get(&edge.target) else {
            continue;
        };
        graph.add_edge(node_indices[*source_idx], node_indices[*target_idx], ());
    }

    let mut adjacency = HashMap::<String, Vec<String>>::new();
    for edge in edges {
        adjacency
            .entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
    }

    let mut visited = HashSet::<String>::new();
    let mut stack = Vec::<String>::new();
    let mut in_stack = HashSet::<String>::new();
    let mut seen_cycles = HashSet::<String>::new();
    let mut cycles = Vec::<Vec<String>>::new();

    for node in nodes {
        dfs_cycle(
            &node.id,
            &adjacency,
            &mut visited,
            &mut stack,
            &mut in_stack,
            &mut seen_cycles,
            &mut cycles,
        );
    }

    cycles
}

fn dfs_cycle(
    node: &str,
    adjacency: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
    in_stack: &mut HashSet<String>,
    seen_cycles: &mut HashSet<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if visited.contains(node) {
        return;
    }

    visited.insert(node.to_string());
    stack.push(node.to_string());
    in_stack.insert(node.to_string());

    if let Some(neighbours) = adjacency.get(node) {
        for neighbour in neighbours {
            if !visited.contains(neighbour) {
                dfs_cycle(
                    neighbour,
                    adjacency,
                    visited,
                    stack,
                    in_stack,
                    seen_cycles,
                    cycles,
                );
            } else if in_stack.contains(neighbour)
                && let Some(pos) = stack.iter().position(|entry| entry == neighbour)
            {
                let mut cycle = stack[pos..].to_vec();
                cycle.push(neighbour.clone());
                let key = cycle.join("->");
                if seen_cycles.insert(key) {
                    cycles.push(cycle);
                }
            }
        }
    }

    stack.pop();
    in_stack.remove(node);
}

fn apply_warning_policy(diagnostics: &mut [Diagnostic], options: ParseOptions) {
    if !(options.strict || options.warnings_as_errors) {
        return;
    }

    for diagnostic in diagnostics {
        if diagnostic.severity == DiagnosticSeverity::Warning {
            diagnostic.severity = DiagnosticSeverity::Error;
        }
    }
}

fn insert_node(nodes: &mut Vec<GraphNode>, seen: &mut HashSet<String>, node: GraphNode) {
    if seen.insert(node.id.clone()) {
        nodes.push(node);
    }
}

fn path_to_display_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn normalise_slug(input: &str) -> String {
    let lowered = input.to_ascii_lowercase();
    let mut out = String::new();
    let mut prev_hyphen = false;

    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_hyphen = false;
            continue;
        }

        if (ch.is_whitespace() || ch == '-') && !prev_hyphen {
            out.push('-');
            prev_hyphen = true;
        }
    }

    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "section".to_string()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn parse_current_specs_directory() {
        let result = parse_specs_directory("specs", ParseOptions::default()).unwrap();
        assert!(!result.specs.is_empty());
        assert!(
            result
                .specs
                .iter()
                .any(|s| s.file_name == "spec-parsing.md")
        );
    }

    #[test]
    fn detect_nested_spec_file() {
        let temp = tempdir().unwrap();
        let specs = temp.path().join("specs");
        fs::create_dir(&specs).unwrap();
        fs::create_dir(specs.join("nested")).unwrap();
        fs::write(specs.join("root.md"), "---\nKind: context\n---\n\n# Root\n").unwrap();
        fs::write(
            specs.join("nested").join("child.md"),
            "---\nKind: context\n---\n\n# Child\n",
        )
        .unwrap();

        let result = parse_specs_directory(&specs, ParseOptions::default()).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == DiagnosticCode::NestedSpecFile)
        );
    }

    #[test]
    fn parse_specifies_edges_and_broken_heading() {
        let temp = tempdir().unwrap();
        let specs = temp.path().join("specs");
        fs::create_dir(&specs).unwrap();
        fs::write(
            specs.join("architecture.md"),
            "---\nKind: feature\n---\n\n# Architecture\n\n## Parser\n",
        )
        .unwrap();
        fs::write(
            specs.join("child.md"),
            "---\nKind: behavioural\nSpecifies:\n  - architecture.md#parser\n  - architecture.md#does-not-exist\n---\n\n# Child\n",
        )
        .unwrap();

        let result = parse_specs_directory(&specs, ParseOptions::default()).unwrap();
        assert!(result.graph.edges.iter().any(|edge| {
            edge.kind == GraphEdgeKind::Specifies
                && edge.source == "child.md"
                && edge.target == "architecture.md#parser"
        }));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == DiagnosticCode::BrokenTargetHeading)
        );
    }

    #[test]
    fn detect_cycle() {
        let temp = tempdir().unwrap();
        let specs = temp.path().join("specs");
        fs::create_dir(&specs).unwrap();
        fs::write(
            specs.join("a.md"),
            "---\nKind: feature\nSpecifies:\n  - b.md#b\n---\n\n# A\n\n## A\n",
        )
        .unwrap();
        fs::write(
            specs.join("b.md"),
            "---\nKind: feature\nSpecifies:\n  - a.md#a\n---\n\n# B\n\n## B\n",
        )
        .unwrap();

        let result = parse_specs_directory(&specs, ParseOptions::default()).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == DiagnosticCode::CycleDetected)
        );
    }

    #[test]
    fn detect_orphan_feature() {
        let temp = tempdir().unwrap();
        let specs = temp.path().join("specs");
        fs::create_dir(&specs).unwrap();
        fs::write(
            specs.join("feature.md"),
            "---\nKind: feature\n---\n\n# Feature\n",
        )
        .unwrap();

        let result = parse_specs_directory(&specs, ParseOptions::default()).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == DiagnosticCode::OrphanSpec)
        );
    }

    proptest! {
        #[test]
        fn slug_normalisation_is_lowercase_and_no_spaces(input in "\\PC*") {
            let slug = normalise_slug(&input);
            prop_assert!(!slug.chars().any(|c| c.is_ascii_uppercase()));
            prop_assert!(!slug.contains(' '));
            prop_assert!(!slug.is_empty());
        }
    }
}
