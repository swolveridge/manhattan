use clap::{Args, Parser, Subcommand};
use manhattan_spec_parser::{Diagnostic, DiagnosticSeverity, ParseOptions, parse_specs_directory};

#[derive(Debug, Parser)]
#[command(name = "manhattan")]
#[command(about = "Spec-driven codebase reconciliation tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Parse(ParseCommand),
}

#[derive(Debug, Args)]
struct ParseCommand {
    #[command(subcommand)]
    command: ParseSubcommand,
}

#[derive(Debug, Subcommand)]
enum ParseSubcommand {
    Check {
        #[arg(default_value = "specs")]
        directory: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Parse(parse) => match parse.command {
            ParseSubcommand::Check { directory } => run_parse_check(&directory),
        },
    }
}

fn run_parse_check(directory: &str) {
    let result = parse_specs_directory(directory, ParseOptions::default());
    match result {
        Ok(parsed) => {
            let warning_count = parsed
                .diagnostics
                .iter()
                .filter(|d| d.severity == DiagnosticSeverity::Warning)
                .count();
            let error_count = parsed
                .diagnostics
                .iter()
                .filter(|d| d.severity == DiagnosticSeverity::Error)
                .count();

            println!("Spec parse summary");
            println!("  directory: {directory}");
            println!("  files parsed: {}", parsed.specs.len());
            println!("  graph nodes: {}", parsed.graph.nodes.len());
            println!("  graph edges: {}", parsed.graph.edges.len());
            println!("  warnings: {warning_count}");
            println!("  errors: {error_count}");

            let mut diagnostics = parsed.diagnostics;
            diagnostics.sort_by_key(diagnostic_sort_key);
            if diagnostics.is_empty() {
                println!();
                println!("No issues found.");
                return;
            }

            println!();
            println!("Issues:");
            for diagnostic in diagnostics {
                println!("  - {}", format_diagnostic(&diagnostic));
            }
        }
        Err(err) => {
            println!("Spec parse summary");
            println!("  directory: {directory}");
            println!("  files parsed: 0");
            println!("  graph nodes: 0");
            println!("  graph edges: 0");
            println!("  warnings: 0");
            println!("  errors: 1");
            println!();
            println!("Issues:");
            println!("  - ERROR IO_READ_FAILURE: {err}");
        }
    }
}

fn diagnostic_sort_key(diagnostic: &Diagnostic) -> (String, usize, usize, String) {
    let file = diagnostic
        .location
        .as_ref()
        .map(|loc| loc.file_name.clone())
        .unwrap_or_default();
    let line = diagnostic
        .location
        .as_ref()
        .and_then(|loc| loc.line)
        .unwrap_or(usize::MAX);
    let column = diagnostic
        .location
        .as_ref()
        .and_then(|loc| loc.column)
        .unwrap_or(usize::MAX);
    (file, line, column, diagnostic.message.clone())
}

fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Warning => "WARNING",
        DiagnosticSeverity::Error => "ERROR",
    };
    let code = format!("{:?}", diagnostic.code).to_uppercase();
    let location = diagnostic
        .location
        .as_ref()
        .map(|loc| match (loc.line, loc.column) {
            (Some(line), Some(column)) => format!("{}:{line}:{column}", loc.file_name),
            (Some(line), None) => format!("{}:{line}", loc.file_name),
            _ => loc.file_name.clone(),
        })
        .unwrap_or_else(|| "<unknown>".to_string());
    format!("{severity} {code} {location} {}", diagnostic.message)
}
