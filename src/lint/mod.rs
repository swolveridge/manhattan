use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::llm::{
    ChatInvoker, ChatRequest, InvocationError, JsonSchema, Message, ResponseFormat, Role,
};
use crate::parse::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, ParseOptions, ParsedSpec, SourceLocation,
    SpecKind, parse_specs_directory,
};

const CONTRADICTION_TEMPLATE: &str = include_str!("prompts/contradiction.md");
const GAP_TEMPLATE: &str = include_str!("prompts/gap.md");
const AMBIGUITY_TEMPLATE: &str = include_str!("prompts/ambiguity.md");

const CONTRADICTION_SCHEMA: &str = r#"{
  "type": "object",
  "additionalProperties": false,
  "required": ["findings"],
  "properties": {
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["is_contradiction", "confidence", "file_a", "file_b", "message", "evidence_a", "evidence_b"],
        "properties": {
          "is_contradiction": {"type": "boolean"},
          "confidence": {"type": "string", "enum": ["high", "medium", "low"]},
          "file_a": {"type": "string"},
          "file_b": {"type": "string"},
          "message": {"type": "string"},
          "evidence_a": {"type": "string"},
          "evidence_b": {"type": "string"}
        }
      }
    }
  }
}"#;

const GAP_SCHEMA: &str = r#"{
  "type": "object",
  "additionalProperties": false,
  "required": ["findings"],
  "properties": {
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["has_gap", "confidence", "message", "location_file", "evidence"],
        "properties": {
          "has_gap": {"type": "boolean"},
          "confidence": {"type": "string", "enum": ["high", "medium", "low"]},
          "message": {"type": "string"},
          "location_file": {"type": "string"},
          "evidence": {"type": "string"}
        }
      }
    }
  }
}"#;

const AMBIGUITY_SCHEMA: &str = r#"{
  "type": "object",
  "additionalProperties": false,
  "required": ["findings"],
  "properties": {
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["confidence", "location_file", "message", "evidence"],
        "properties": {
          "confidence": {"type": "string", "enum": ["high", "medium", "low"]},
          "location_file": {"type": "string"},
          "message": {"type": "string"},
          "evidence": {"type": "string"}
        }
      }
    }
  }
}"#;

const SPEC_FILE_EXEMPTION: &str = "spec-format.md";

#[derive(Debug, Clone)]
pub struct LintOptions {
    pub structural_only: bool,
    pub focus: Option<String>,
    pub llm_model: String,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            structural_only: false,
            focus: None,
            llm_model: "anthropic/claude-sonnet-4.6".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintCategory {
    Structural,
    Contradiction,
    Gap,
    Ambiguity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintIssue {
    pub severity: LintSeverity,
    pub category: LintCategory,
    pub code: String,
    pub message: String,
    pub location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintResult {
    pub issues: Vec<LintIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCheck {
    Contradiction,
    Gap,
    Ambiguity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticPrompt {
    pub check: SemanticCheck,
    pub prompt: String,
}

#[derive(Debug, Error)]
pub enum LintError {
    #[error("failed to parse specs: {0}")]
    Parse(#[from] crate::parse::ParserError),
    #[error("semantic linting requested without an LLM invoker")]
    MissingInvoker,
    #[error(
        "invalid --focus path '{focus}' (must be relative to specs directory and end with .md)"
    )]
    InvalidFocus { focus: String },
    #[error("focused spec '{focus}' was not found in parsed corpus")]
    FocusNotFound { focus: String },
    #[error("failed reading '{path}': {source}")]
    ReadSpec {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("LLM invocation failed: {0}")]
    Invocation(#[from] InvocationError),
    #[error("invalid semantic response JSON for {check}: {source}")]
    SemanticJson {
        check: &'static str,
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone)]
struct SpecDocument {
    file_name: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ContradictionPayload {
    findings: Vec<ContradictionFinding>,
}

#[derive(Debug, Deserialize)]
struct ContradictionFinding {
    is_contradiction: bool,
    confidence: Confidence,
    file_a: String,
    file_b: String,
    message: String,
    evidence_a: String,
    evidence_b: String,
}

#[derive(Debug, Deserialize)]
struct GapPayload {
    findings: Vec<GapFinding>,
}

#[derive(Debug, Deserialize)]
struct GapFinding {
    has_gap: bool,
    confidence: Confidence,
    message: String,
    location_file: String,
    evidence: String,
}

#[derive(Debug, Deserialize)]
struct AmbiguityPayload {
    findings: Vec<AmbiguityFinding>,
}

#[derive(Debug, Deserialize)]
struct AmbiguityFinding {
    confidence: Confidence,
    location_file: String,
    message: String,
    evidence: String,
}

pub fn lint_specs_directory_structural(
    specs_dir: impl AsRef<Path>,
    options: LintOptions,
) -> Result<LintResult, LintError> {
    let parsed = parse_specs_directory(specs_dir, ParseOptions::default())?;
    let scope = resolve_scope(&parsed.specs, options.focus.as_deref())?;

    let mut issues = map_parser_diagnostics(&parsed.diagnostics, &scope);
    issues.extend(policy_checks(&parsed.specs, &scope));
    sort_issues(&mut issues);
    dedupe_issues(&mut issues);

    Ok(LintResult { issues })
}

pub fn lint_semantic_prompts_for_directory(
    specs_dir: impl AsRef<Path>,
    options: LintOptions,
) -> Result<Vec<SemanticPrompt>, LintError> {
    if options.structural_only {
        return Ok(Vec::new());
    }

    let specs_dir = specs_dir.as_ref();
    let parsed = parse_specs_directory(specs_dir, ParseOptions::default())?;
    let scope = resolve_scope(&parsed.specs, options.focus.as_deref())?;
    let docs = load_documents(specs_dir, &parsed.specs, &scope)?;
    let index = docs
        .iter()
        .map(|doc| doc.file_name.clone())
        .collect::<Vec<_>>();
    let corpus = build_spec_corpus(&docs);

    Ok(vec![
        SemanticPrompt {
            check: SemanticCheck::Contradiction,
            prompt: fill_template(
                CONTRADICTION_TEMPLATE,
                &[("SPEC_INDEX", &index.join("\n")), ("SPEC_CORPUS", &corpus)],
            ),
        },
        SemanticPrompt {
            check: SemanticCheck::Gap,
            prompt: fill_template(
                GAP_TEMPLATE,
                &[("SPEC_INDEX", &index.join("\n")), ("SPEC_CORPUS", &corpus)],
            ),
        },
        SemanticPrompt {
            check: SemanticCheck::Ambiguity,
            prompt: fill_template(
                AMBIGUITY_TEMPLATE,
                &[("SPEC_INDEX", &index.join("\n")), ("SPEC_CORPUS", &corpus)],
            ),
        },
    ])
}

pub async fn lint_specs_directory_with_invoker<I>(
    specs_dir: impl AsRef<Path>,
    options: LintOptions,
    invoker: Option<&I>,
) -> Result<LintResult, LintError>
where
    I: ChatInvoker + Sync,
{
    let specs_dir = specs_dir.as_ref();
    let parsed = parse_specs_directory(specs_dir, ParseOptions::default())?;
    let scope = resolve_scope(&parsed.specs, options.focus.as_deref())?;

    let mut issues = map_parser_diagnostics(&parsed.diagnostics, &scope);
    issues.extend(policy_checks(&parsed.specs, &scope));

    if !options.structural_only {
        let invoker = invoker.ok_or(LintError::MissingInvoker)?;
        let docs = load_documents(specs_dir, &parsed.specs, &scope)?;
        let index = docs
            .iter()
            .map(|doc| doc.file_name.clone())
            .collect::<Vec<_>>();

        issues.extend(run_contradiction_checks(invoker, &options.llm_model, &docs, &index).await?);
        issues.extend(run_gap_check(invoker, &options.llm_model, &docs, &index).await?);
        issues.extend(run_ambiguity_checks(invoker, &options.llm_model, &docs, &index).await?);
    }

    sort_issues(&mut issues);
    dedupe_issues(&mut issues);
    Ok(LintResult { issues })
}

fn map_parser_diagnostics(diagnostics: &[Diagnostic], scope: &BTreeSet<String>) -> Vec<LintIssue> {
    diagnostics
        .iter()
        .filter(|diag| parser_code_is_graph_integrity(diag.code))
        .filter(|diag| diagnostic_in_scope(diag, scope))
        .map(|diag| LintIssue {
            severity: diagnostic_severity_to_lint(diag.severity),
            category: LintCategory::Structural,
            code: format!("STRUCT_{:?}", diag.code).to_ascii_uppercase(),
            message: diag.message.clone(),
            location: diag.location.clone(),
            confidence: None,
            evidence: Vec::new(),
        })
        .collect()
}

fn parser_code_is_graph_integrity(code: DiagnosticCode) -> bool {
    matches!(
        code,
        DiagnosticCode::IoReadFailure
            | DiagnosticCode::InvalidFrontMatter
            | DiagnosticCode::InvalidKind
            | DiagnosticCode::InvalidRoot
            | DiagnosticCode::InvalidSpecifies
            | DiagnosticCode::InvalidSpecifiesTarget
            | DiagnosticCode::InvalidCrossReference
            | DiagnosticCode::BrokenTargetFile
            | DiagnosticCode::BrokenTargetHeading
            | DiagnosticCode::BrokenCrossReferenceFile
            | DiagnosticCode::BrokenCrossReferenceHeading
            | DiagnosticCode::CycleDetected
    )
}

fn diagnostic_severity_to_lint(severity: DiagnosticSeverity) -> LintSeverity {
    match severity {
        DiagnosticSeverity::Warning => LintSeverity::Warning,
        DiagnosticSeverity::Error => LintSeverity::Error,
    }
}

fn policy_checks(specs: &[ParsedSpec], scope: &BTreeSet<String>) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let scoped_specs = specs
        .iter()
        .filter(|spec| scope.contains(&spec.file_name))
        .collect::<Vec<_>>();

    let root_specs = scoped_specs
        .iter()
        .filter(|spec| spec.is_root)
        .collect::<Vec<_>>();

    if root_specs.len() > 1 {
        for spec in &root_specs {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                category: LintCategory::Structural,
                code: "STRUCT_ROOT_MULTIPLE".to_string(),
                message: format!(
                    "Declare Root: true in at most one spec; duplicate root in '{}'.",
                    spec.file_name
                ),
                location: Some(SourceLocation {
                    file_name: spec.file_name.clone(),
                    line: Some(1),
                    column: Some(1),
                }),
                confidence: None,
                evidence: Vec::new(),
            });
        }
    }

    for spec in &root_specs {
        if spec.kind != Some(SpecKind::Feature) {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                category: LintCategory::Structural,
                code: "STRUCT_ROOT_KIND".to_string(),
                message: format!(
                    "Use Root: true only on Kind: feature specs in '{}'.",
                    spec.file_name
                ),
                location: Some(SourceLocation {
                    file_name: spec.file_name.clone(),
                    line: Some(1),
                    column: Some(1),
                }),
                confidence: None,
                evidence: Vec::new(),
            });
        }
    }

    let valid_outgoing = scoped_specs
        .iter()
        .map(|spec| {
            (
                spec.file_name.clone(),
                spec.specifies
                    .iter()
                    .filter(|target| scope.contains(&target.file_name))
                    .count(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for spec in scoped_specs {
        if spec.headings.is_empty() {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                category: LintCategory::Structural,
                code: "STRUCT_MISSING_HEADING".to_string(),
                message: format!(
                    "Add at least one markdown heading to provide required description structure in '{}'.",
                    spec.file_name
                ),
                location: Some(SourceLocation {
                    file_name: spec.file_name.clone(),
                    line: Some(1),
                    column: Some(1),
                }),
                confidence: None,
                evidence: Vec::new(),
            });
        }

        if spec.file_name == SPEC_FILE_EXEMPTION {
            continue;
        }

        if !matches!(
            spec.kind,
            Some(SpecKind::Feature | SpecKind::Behavioural | SpecKind::Interface)
        ) {
            continue;
        }

        if spec.is_root {
            continue;
        }

        if valid_outgoing.get(&spec.file_name).copied().unwrap_or(0) == 0 {
            issues.push(LintIssue {
                severity: LintSeverity::Warning,
                category: LintCategory::Structural,
                code: "STRUCT_ORPHAN_SPEC".to_string(),
                message: format!(
                    "Add at least one valid Specifies target for '{}', or change Kind if it should float.",
                    spec.file_name
                ),
                location: Some(SourceLocation {
                    file_name: spec.file_name.clone(),
                    line: Some(1),
                    column: Some(1),
                }),
                confidence: None,
                evidence: Vec::new(),
            });
        }
    }

    issues
}

fn resolve_scope(specs: &[ParsedSpec], focus: Option<&str>) -> Result<BTreeSet<String>, LintError> {
    let all_files = specs
        .iter()
        .map(|spec| spec.file_name.clone())
        .collect::<BTreeSet<_>>();

    let Some(focus) = focus else {
        return Ok(all_files);
    };

    let focus = normalize_focus_path(focus).ok_or_else(|| LintError::InvalidFocus {
        focus: focus.to_string(),
    })?;

    if !all_files.contains(&focus) {
        return Err(LintError::FocusNotFound { focus });
    }

    let mut scope = BTreeSet::from([focus.clone()]);

    for spec in specs {
        if spec.file_name == focus {
            for target in &spec.specifies {
                scope.insert(target.file_name.clone());
            }
        }

        if spec
            .specifies
            .iter()
            .any(|target| target.file_name == focus)
        {
            scope.insert(spec.file_name.clone());
        }
    }

    Ok(scope)
}

fn normalize_focus_path(focus: &str) -> Option<String> {
    let normalized = focus.trim().replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || !normalized.to_ascii_lowercase().ends_with(".md")
    {
        None
    } else {
        Some(normalized)
    }
}

fn diagnostic_in_scope(diagnostic: &Diagnostic, scope: &BTreeSet<String>) -> bool {
    match &diagnostic.location {
        Some(location) => scope.contains(&location.file_name),
        None => true,
    }
}

fn load_documents(
    specs_dir: &Path,
    specs: &[ParsedSpec],
    scope: &BTreeSet<String>,
) -> Result<Vec<SpecDocument>, LintError> {
    let mut documents = Vec::new();
    for spec in specs {
        if !scope.contains(&spec.file_name) {
            continue;
        }
        let path = specs_dir.join(&spec.file_name);
        let content = fs::read_to_string(&path).map_err(|source| LintError::ReadSpec {
            path: path.clone(),
            source,
        })?;
        documents.push(SpecDocument {
            file_name: spec.file_name.clone(),
            content,
        });
    }
    Ok(documents)
}

async fn run_contradiction_checks<I>(
    invoker: &I,
    model: &str,
    docs: &[SpecDocument],
    index: &[String],
) -> Result<Vec<LintIssue>, LintError>
where
    I: ChatInvoker + Sync,
{
    let corpus = build_spec_corpus(docs);
    let prompt = fill_template(
        CONTRADICTION_TEMPLATE,
        &[("SPEC_INDEX", &index.join("\n")), ("SPEC_CORPUS", &corpus)],
    );

    let payload: ContradictionPayload = invoke_structured_json(
        invoker,
        model,
        "contradiction_detection",
        CONTRADICTION_SCHEMA,
        prompt,
        "contradiction",
    )
    .await?;

    let mut issues = Vec::new();
    for finding in payload.findings {
        if !finding.is_contradiction {
            continue;
        }
        issues.push(LintIssue {
            severity: LintSeverity::Error,
            category: LintCategory::Contradiction,
            code: "SEM_CONTRADICTION".to_string(),
            message: finding.message,
            location: Some(SourceLocation {
                file_name: finding.file_a.clone(),
                line: None,
                column: None,
            }),
            confidence: Some(finding.confidence),
            evidence: vec![
                format!("{}: {}", finding.file_a, finding.evidence_a),
                format!("{}: {}", finding.file_b, finding.evidence_b),
            ],
        });
    }

    Ok(issues)
}

async fn run_gap_check<I>(
    invoker: &I,
    model: &str,
    docs: &[SpecDocument],
    index: &[String],
) -> Result<Vec<LintIssue>, LintError>
where
    I: ChatInvoker + Sync,
{
    let corpus = build_spec_corpus(docs);

    let prompt = fill_template(
        GAP_TEMPLATE,
        &[("SPEC_INDEX", &index.join("\n")), ("SPEC_CORPUS", &corpus)],
    );

    let payload: GapPayload =
        invoke_structured_json(invoker, model, "gap_detection", GAP_SCHEMA, prompt, "gap").await?;

    let mut issues = Vec::new();
    for finding in payload.findings {
        if !finding.has_gap {
            continue;
        }
        let location = if finding.location_file.is_empty() {
            None
        } else {
            Some(SourceLocation {
                file_name: finding.location_file.clone(),
                line: None,
                column: None,
            })
        };

        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            category: LintCategory::Gap,
            code: "SEM_GAP".to_string(),
            message: finding.message,
            location,
            confidence: Some(finding.confidence),
            evidence: vec![finding.evidence],
        });
    }

    Ok(issues)
}

async fn run_ambiguity_checks<I>(
    invoker: &I,
    model: &str,
    docs: &[SpecDocument],
    index: &[String],
) -> Result<Vec<LintIssue>, LintError>
where
    I: ChatInvoker + Sync,
{
    let corpus = build_spec_corpus(docs);
    let prompt = fill_template(
        AMBIGUITY_TEMPLATE,
        &[("SPEC_INDEX", &index.join("\n")), ("SPEC_CORPUS", &corpus)],
    );

    let payload: AmbiguityPayload = invoke_structured_json(
        invoker,
        model,
        "ambiguity_detection",
        AMBIGUITY_SCHEMA,
        prompt,
        "ambiguity",
    )
    .await?;

    let mut issues = Vec::new();
    for finding in payload.findings {
        let location = if finding.location_file.is_empty() {
            None
        } else {
            Some(SourceLocation {
                file_name: finding.location_file.clone(),
                line: None,
                column: None,
            })
        };

        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            category: LintCategory::Ambiguity,
            code: "SEM_AMBIGUITY".to_string(),
            message: finding.message,
            location,
            confidence: Some(finding.confidence),
            evidence: vec![finding.evidence],
        });
    }

    Ok(issues)
}

fn build_spec_corpus(docs: &[SpecDocument]) -> String {
    let mut corpus = String::new();
    for doc in docs {
        corpus.push_str("=== FILE: ");
        corpus.push_str(&doc.file_name);
        corpus.push_str(" ===\n");
        corpus.push_str(&doc.content);
        corpus.push_str("\n\n");
    }
    corpus
}

async fn invoke_structured_json<I, T>(
    invoker: &I,
    model: &str,
    schema_name: &str,
    schema: &str,
    prompt: String,
    check: &'static str,
) -> Result<T, LintError>
where
    I: ChatInvoker + Sync,
    T: for<'de> Deserialize<'de>,
{
    let schema_value =
        serde_json::from_str(schema).map_err(|source| LintError::SemanticJson { check, source })?;

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: Role::User,
            content: Some(prompt),
            tool_call_id: None,
            tool_calls: None,
        }],
        response_format: Some(ResponseFormat {
            format_type: "json_schema".to_string(),
            json_schema: Some(JsonSchema {
                name: schema_name.to_string(),
                schema: schema_value,
                strict: Some(true),
            }),
        }),
        tools: None,
        tool_choice: None,
        temperature: Some(0.0),
        max_tokens: Some(5_000),
        reasoning_effort: Some("medium".to_string()),
    };

    let response = invoker.chat(request).await?;
    let content = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or_else(|| InvocationError::InvalidResponse("missing assistant content".to_string()))?
        .trim()
        .to_string();

    serde_json::from_str::<T>(&content).map_err(|source| LintError::SemanticJson { check, source })
}

fn fill_template(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in replacements {
        let key = format!("{{{{{name}}}}}");
        out = out.replace(&key, value);
    }
    out
}

fn sort_issues(issues: &mut [LintIssue]) {
    issues.sort_by_key(|issue| {
        let file = issue
            .location
            .as_ref()
            .map(|location| location.file_name.clone())
            .unwrap_or_default();
        let line = issue
            .location
            .as_ref()
            .and_then(|location| location.line)
            .unwrap_or(usize::MAX);
        let column = issue
            .location
            .as_ref()
            .and_then(|location| location.column)
            .unwrap_or(usize::MAX);
        (
            file,
            line,
            column,
            format!("{:?}", issue.category),
            issue.code.clone(),
            issue.message.clone(),
        )
    });
}

fn dedupe_issues(issues: &mut Vec<LintIssue>) {
    let mut deduped = Vec::with_capacity(issues.len());
    let mut seen = BTreeSet::new();
    for issue in issues.drain(..) {
        let key = (
            issue.category.clone(),
            issue.code.clone(),
            issue.message.clone(),
            issue
                .location
                .as_ref()
                .map(|location| {
                    (
                        location.file_name.clone(),
                        location.line.unwrap_or(usize::MAX),
                        location.column.unwrap_or(usize::MAX),
                    )
                })
                .unwrap_or_else(|| (String::new(), usize::MAX, usize::MAX)),
        );
        if seen.insert(key) {
            deduped.push(issue);
        }
    }
    *issues = deduped;
}

pub fn summarize_severities(issues: &[LintIssue]) -> (usize, usize) {
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == LintSeverity::Warning)
        .count();
    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == LintSeverity::Error)
        .count();
    (warning_count, error_count)
}

pub fn suggested_exit_code(issues: &[LintIssue]) -> i32 {
    let (warnings, errors) = summarize_severities(issues);
    if errors > 0 {
        2
    } else if warnings > 0 {
        1
    } else {
        0
    }
}

pub fn format_issue(issue: &LintIssue) -> String {
    let severity = match issue.severity {
        LintSeverity::Warning => "WARNING",
        LintSeverity::Error => "ERROR",
    };
    let location = issue
        .location
        .as_ref()
        .map(|location| match (location.line, location.column) {
            (Some(line), Some(column)) => format!("{}:{line}:{column}", location.file_name),
            (Some(line), None) => format!("{}:{line}", location.file_name),
            _ => location.file_name.clone(),
        })
        .unwrap_or_else(|| "<unknown>".to_string());

    let mut suffix = String::new();
    if let Some(confidence) = issue.confidence {
        let value = match confidence {
            Confidence::High => "high",
            Confidence::Medium => "medium",
            Confidence::Low => "low",
        };
        suffix.push_str(&format!(" [confidence={value}]"));
    }

    format!(
        "{severity} {} {:?} {location} {}{}",
        issue.code, issue.category, issue.message, suffix
    )
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::future::Future;
    use std::sync::Mutex;

    use super::*;
    use crate::llm::{ChatResponse, Choice};
    use serde_json::json;
    use tempfile::tempdir;

    struct FakeInvoker {
        responses: Mutex<VecDeque<String>>,
    }

    impl ChatInvoker for FakeInvoker {
        fn chat(
            &self,
            request: ChatRequest,
        ) -> impl Future<Output = Result<ChatResponse, InvocationError>> + Send {
            let prompt = request
                .messages
                .first()
                .and_then(|message| message.content.clone())
                .unwrap_or_default();

            let content = if let Some(next) = self.responses.lock().expect("lock").pop_front() {
                next
            } else if prompt.contains("Role: You are a specification contradiction analyst.") {
                if prompt.contains("spec must require authentication")
                    && prompt.contains("spec must not require authentication")
                {
                    json!({
                        "findings": [{
                            "is_contradiction": true,
                            "confidence": "high",
                            "file_a": "auth-required.md",
                            "file_b": "auth-not-required.md",
                            "message": "Auth requirement conflicts.",
                            "evidence_a": "must require authentication",
                            "evidence_b": "must not require authentication"
                        }]
                    })
                    .to_string()
                } else {
                    json!({"findings": []}).to_string()
                }
            } else if prompt.contains("Role: You are a specification ambiguity analyst.")
                && prompt.contains("quickly")
            {
                json!({
                    "findings": [{
                        "confidence": "high",
                        "location_file": "ambiguous.md",
                        "message": "\"quickly\" is vague.",
                        "evidence": "The system should respond quickly."
                    }]
                })
                .to_string()
            } else if prompt.contains("Role: You are a specification gap analyst.") {
                json!({"findings": []}).to_string()
            } else {
                json!({"findings": []}).to_string()
            };

            let response = ChatResponse {
                id: "fake-1".to_string(),
                model: "openai/gpt-5".to_string(),
                choices: vec![Choice {
                    index: 0,
                    message: Message {
                        role: Role::Assistant,
                        content: Some(content),
                        tool_call_id: None,
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: None,
            };

            async move { Ok(response) }
        }
    }

    #[test]
    fn structural_flags_missing_heading_and_orphan() {
        let temp = tempdir().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir(&specs_dir).unwrap();

        fs::write(
            specs_dir.join("feature.md"),
            "---\nKind: feature\n---\n\nNo headings here.\n",
        )
        .unwrap();

        let result = lint_specs_directory_structural(&specs_dir, LintOptions::default()).unwrap();

        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == "STRUCT_MISSING_HEADING")
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.code == "STRUCT_ORPHAN_SPEC")
        );
    }

    #[test]
    fn structural_focus_limits_scope() {
        let temp = tempdir().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir(&specs_dir).unwrap();

        fs::write(
            specs_dir.join("architecture.md"),
            "---\nKind: feature\nRoot: true\n---\n\n# Architecture\n\n## Root\n",
        )
        .unwrap();
        fs::write(
            specs_dir.join("a.md"),
            "---\nKind: behavioural\nSpecifies:\n  - architecture.md#root\n---\n\n# A\n",
        )
        .unwrap();
        fs::write(
            specs_dir.join("unrelated.md"),
            "---\nKind: feature\n---\n\n# Unrelated\n",
        )
        .unwrap();

        let result = lint_specs_directory_structural(
            &specs_dir,
            LintOptions {
                focus: Some("a.md".to_string()),
                ..LintOptions::default()
            },
        )
        .unwrap();

        assert!(result.issues.iter().all(|issue| {
            issue
                .location
                .as_ref()
                .is_none_or(|location| location.file_name != "unrelated.md")
        }));
    }

    #[tokio::test]
    async fn semantic_contradiction_and_ambiguity_detected() {
        let temp = tempdir().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir(&specs_dir).unwrap();

        fs::write(
            specs_dir.join("architecture.md"),
            "---\nKind: feature\nRoot: true\n---\n\n# Architecture\n\n## Auth\n",
        )
        .unwrap();

        fs::write(
            specs_dir.join("auth-required.md"),
            "---\nKind: behavioural\nSpecifies:\n  - architecture.md#auth\n---\n\n# Required\n\nThe spec must require authentication.\n",
        )
        .unwrap();

        fs::write(
            specs_dir.join("auth-not-required.md"),
            "---\nKind: behavioural\nSpecifies:\n  - architecture.md#auth\n---\n\n# Not Required\n\nThe spec must not require authentication.\n",
        )
        .unwrap();

        fs::write(
            specs_dir.join("ambiguous.md"),
            "---\nKind: behavioural\nSpecifies:\n  - architecture.md#auth\n---\n\n# Ambiguous\n\nThe system should respond quickly.\n",
        )
        .unwrap();

        let invoker = FakeInvoker {
            responses: Mutex::new(VecDeque::new()),
        };

        let result =
            lint_specs_directory_with_invoker(&specs_dir, LintOptions::default(), Some(&invoker))
                .await
                .unwrap();

        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.category == LintCategory::Contradiction)
        );
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.category == LintCategory::Ambiguity)
        );
    }

    #[test]
    fn exit_code_matches_severity() {
        let ok = Vec::<LintIssue>::new();
        assert_eq!(suggested_exit_code(&ok), 0);

        let warnings = vec![LintIssue {
            severity: LintSeverity::Warning,
            category: LintCategory::Structural,
            code: "STRUCT_ORPHAN_SPEC".to_string(),
            message: "warning".to_string(),
            location: None,
            confidence: None,
            evidence: Vec::new(),
        }];
        assert_eq!(suggested_exit_code(&warnings), 1);

        let errors = vec![LintIssue {
            severity: LintSeverity::Error,
            category: LintCategory::Structural,
            code: "STRUCT_MISSING_HEADING".to_string(),
            message: "error".to_string(),
            location: None,
            confidence: None,
            evidence: Vec::new(),
        }];
        assert_eq!(suggested_exit_code(&errors), 2);
    }
}
