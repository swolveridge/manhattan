use std::process;

use clap::{Args, Parser, Subcommand};
use manhattan_spec_parser::{
    ClientConfig, LintIssue, LintOptions, OpenAiCompatibleClient, format_issue,
    lint_semantic_prompts_for_directory, lint_specs_directory_structural,
    lint_specs_directory_with_invoker, suggested_exit_code, summarize_severities,
};

#[derive(Debug, Parser)]
#[command(name = "manhattan")]
#[command(about = "Spec-driven codebase reconciliation tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Lint(LintCommand),
}

#[derive(Debug, Args)]
struct LintCommand {
    #[command(subcommand)]
    command: LintSubcommand,
}

#[derive(Debug, Subcommand)]
enum LintSubcommand {
    Check {
        #[arg(default_value = "specs")]
        directory: String,
        #[arg(long)]
        structural_only: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        focus: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "anthropic/claude-sonnet-4.6")]
        model: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Lint(lint) => match lint.command {
            LintSubcommand::Check {
                directory,
                structural_only,
                dry_run,
                focus,
                json,
                model,
            } => run_lint_check(&directory, structural_only, dry_run, focus, json, &model),
        },
    }
}

fn run_lint_check(
    directory: &str,
    structural_only: bool,
    dry_run: bool,
    focus: Option<String>,
    json_output: bool,
    model: &str,
) {
    let options = LintOptions {
        structural_only,
        focus,
        llm_model: model.to_string(),
    };

    if dry_run {
        match lint_semantic_prompts_for_directory(directory, options.clone()) {
            Ok(prompts) => emit_lint_dry_run(directory, model, prompts, json_output),
            Err(err) => {
                eprintln!("Lint failed: {err}");
                process::exit(2);
            }
        }
        return;
    }

    if structural_only {
        match lint_specs_directory_structural(directory, options) {
            Ok(result) => emit_lint_result(directory, result.issues, json_output),
            Err(err) => {
                eprintln!("Lint failed: {err}");
                process::exit(2);
            }
        }
        return;
    }

    let client = match OpenAiCompatibleClient::from_config(ClientConfig::default()) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Lint failed: {err}");
            process::exit(2);
        }
    };

    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let result = runtime.block_on(lint_specs_directory_with_invoker(
        directory,
        options,
        Some(&client),
    ));

    match result {
        Ok(result) => emit_lint_result(directory, result.issues, json_output),
        Err(err) => {
            eprintln!("Lint failed: {err}");
            process::exit(2);
        }
    }
}

fn emit_lint_result(directory: &str, issues: Vec<LintIssue>, json_output: bool) {
    let (warnings, errors) = summarize_severities(&issues);

    if json_output {
        let payload = serde_json::json!({
            "directory": directory,
            "summary": {
                "warnings": warnings,
                "errors": errors,
                "total": issues.len()
            },
            "issues": issues
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).expect("serialize lint output")
        );
    } else {
        println!("Spec lint summary");
        println!("  directory: {directory}");
        println!("  issues: {}", issues.len());
        println!("  warnings: {warnings}");
        println!("  errors: {errors}");

        if issues.is_empty() {
            println!();
            println!("No issues found.");
        } else {
            println!();
            println!("Issues:");
            for issue in &issues {
                println!("  - {}", format_issue(issue));
                if !issue.evidence.is_empty() {
                    for evidence in &issue.evidence {
                        println!("      evidence: {evidence}");
                    }
                }
            }
        }
    }

    process::exit(suggested_exit_code(&issues));
}

fn emit_lint_dry_run(
    directory: &str,
    model: &str,
    prompts: Vec<manhattan_spec_parser::SemanticPrompt>,
    json_output: bool,
) {
    if json_output {
        let payload = serde_json::json!({
            "directory": directory,
            "model": model,
            "dry_run": true,
            "prompt_count": prompts.len(),
            "prompts": prompts
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).expect("serialize lint dry-run output")
        );
    } else {
        println!("Spec lint dry-run");
        println!("  directory: {directory}");
        println!("  model: {model}");
        println!("  semantic prompts: {}", prompts.len());
        println!();

        if prompts.is_empty() {
            println!("No semantic prompts to print.");
        } else {
            for prompt in prompts {
                let check = match prompt.check {
                    manhattan_spec_parser::SemanticCheck::Contradiction => "contradiction",
                    manhattan_spec_parser::SemanticCheck::Gap => "gap",
                    manhattan_spec_parser::SemanticCheck::Ambiguity => "ambiguity",
                };
                println!("=== {check} prompt ===");
                println!("{}", prompt.prompt);
                println!("=== end {check} prompt ===");
                println!();
            }
        }
    }

    process::exit(0);
}
