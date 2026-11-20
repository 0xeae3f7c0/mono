use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commit_report::Report;

/// commit — self-maintaining repository toolkit
#[derive(Parser)]
#[command(name = "commit", version, about)]
struct Cli {
    /// Repository root (defaults to $GITHUB_WORKSPACE, then the current directory)
    #[arg(long, global = true)]
    path: Option<PathBuf>,

    /// Print the report as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Log progress to stderr (-v debug, -vv trace); RUST_LOG overrides
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Validate required project files, the Cargo manifest, workflows and Git whitespace
    Check,
    /// List changed files relative to BASE_REF and validate repository structure
    Review {
        #[arg(default_value = "origin/main")]
        base_ref: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_logging(cli.verbose);
    let root = cli.path.clone().unwrap_or_else(default_root);

    let report = match &cli.command {
        Cmd::Check => check(&root),
        Cmd::Review { base_ref } => review(&root, base_ref),
    };

    match report {
        Ok(report) => {
            if cli.json {
                println!("{}", report.to_json());
            } else {
                let (out, err) = report.to_text();
                eprint!("{err}");
                print!("{out}");
            }
            if report.ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn init_logging(verbose: u8) {
    let default = match verbose {
        0 => "warn",
        1 => "debug",
        _ => "trace",
    };
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();
}

fn default_root() -> PathBuf {
    env::var_os("GITHUB_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[tracing::instrument]
fn check(path: &Path) -> Result<Report> {
    let mut report = commit_core::validate(path);
    tracing::debug!(
        workflows = report.workflows.len(),
        errors = report.errors.len(),
        "validated repository"
    );
    report.warnings.extend(commit_git::whitespace_warning(path));
    Ok(report)
}

#[tracing::instrument]
fn review(path: &Path, base: &str) -> Result<Report> {
    let mut report = commit_core::validate(path);
    if report.ok() {
        report.changed_files = commit_git::changed_files(path, base)?;
        tracing::debug!(changed = report.changed_files.len(), "compared with base");
    }
    Ok(report)
}
