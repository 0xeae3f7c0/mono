//! Repository validation rules: required files, Cargo manifest and workflows.

use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use commit_report::Report;
use regex::Regex;
use serde::Deserialize;
use walkdir::WalkDir;

pub const REQUIRED: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    ".github/workflows/ci.yml",
    ".github/workflows/maintenance.yml",
    ".github/workflows/code-review.yml",
    ".github/dependabot.yml",
    "crates/cli/src/main.rs",
];

#[derive(Deserialize)]
struct Manifest {
    package: Option<Package>,
    workspace: Option<Workspace>,
}

#[derive(Deserialize)]
struct Workspace {
    package: Option<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: Option<toml::Value>,
    version: Option<toml::Value>,
    license: Option<toml::Value>,
}

pub fn validate(path: &Path) -> Report {
    let mut report = Report::default();

    for file in REQUIRED {
        if !path.join(file).is_file() {
            report.errors.push(format!("Missing required file: {file}"));
        }
    }

    if let Err(error) = validate_manifest(&path.join("Cargo.toml")) {
        report.errors.push(format!("Invalid Cargo.toml: {error:#}"));
    }

    let workflows = path.join(".github/workflows");
    for entry in WalkDir::new(&workflows).into_iter().filter_map(Result::ok) {
        let file = entry.path();
        let is_yaml = matches!(
            file.extension().and_then(|e| e.to_str()),
            Some("yml" | "yaml")
        );
        if !entry.file_type().is_file() || !is_yaml {
            continue;
        }
        let name = file
            .strip_prefix(path)
            .unwrap_or(file)
            .display()
            .to_string()
            .replace('\\', "/");
        match std::fs::read_to_string(file) {
            Ok(text) => check_workflow(&name, &text, &mut report),
            Err(error) => report.errors.push(format!("Cannot read {name}: {error}")),
        }
        report.workflows.push(name);
    }
    report.workflows.sort();
    report
}

/// Parses a workflow as YAML, requires top-level `on:` and `jobs:` keys, and
/// warns about `uses:` references that are not pinned to a version.
fn check_workflow(name: &str, text: &str, report: &mut Report) {
    static UNPINNED: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?m)^\s*-?\s*uses:\s*([^\s#@]+)\s*(?:#.*)?$").expect("valid regex")
    });

    let doc: serde_yaml_ng::Value = match serde_yaml_ng::from_str(text) {
        Ok(doc) => doc,
        Err(error) => {
            report
                .errors
                .push(format!("Invalid YAML in {name}: {error}"));
            return;
        }
    };
    if doc.get("on").is_none() {
        report
            .errors
            .push(format!("Workflow has no top-level `on:` trigger: {name}"));
    }
    if doc.get("jobs").is_none() {
        report
            .errors
            .push(format!("Workflow has no top-level `jobs:` table: {name}"));
    }
    for action in UNPINNED.captures_iter(text).map(|c| c[1].to_owned()) {
        if !action.starts_with("./") && !action.starts_with("docker://") {
            report.warnings.push(format!(
                "Unpinned action `{action}` in {name}; add @<version>"
            ));
        }
    }
}

/// A root manifest is valid when it is a package with name, version and
/// license, or a workspace whose `[workspace.package]` sets version and license.
fn validate_manifest(file: &Path) -> Result<()> {
    if !file.is_file() {
        return Ok(()); // already reported as a missing required file
    }
    let text = std::fs::read_to_string(file).context("cannot read file")?;
    let manifest: Manifest = toml::from_str(&text).context("cannot parse TOML")?;
    let (package, fields): (_, &[&str]) = match (manifest.package, manifest.workspace) {
        (Some(package), _) => (package, &["name", "version", "license"]),
        (
            None,
            Some(Workspace {
                package: Some(package),
            }),
        ) => (package, &["version", "license"]),
        _ => bail!("missing [package] or [workspace.package] table"),
    };
    for field in fields {
        let value = match *field {
            "name" => &package.name,
            "version" => &package.version,
            _ => &package.license,
        };
        if value
            .as_ref()
            .and_then(toml::Value::as_str)
            .is_none_or(str::is_empty)
        {
            bail!("missing {field}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(text: &str) -> Result<()> {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("Cargo.toml");
        std::fs::write(&file, text).unwrap();
        validate_manifest(&file)
    }

    #[test]
    fn required_files_include_core_files() {
        assert!(REQUIRED.contains(&"Cargo.toml"));
        assert!(REQUIRED.contains(&"crates/cli/src/main.rs"));
    }

    #[test]
    fn missing_repository_files_are_reported() {
        let report = validate(Path::new("not-a-real-repository"));
        assert!(report.errors.iter().any(|e| e.contains("Cargo.toml")));
    }

    #[test]
    fn package_manifest_needs_name() {
        assert!(manifest("[package]\nversion = \"1\"\nlicense = \"MIT\"\n").is_err());
        assert!(manifest("[package]\nname = \"a\"\nversion = \"1\"\nlicense = \"MIT\"\n").is_ok());
    }

    fn workflow(text: &str) -> Report {
        let mut report = Report::default();
        check_workflow("w.yml", text, &mut report);
        report
    }

    #[test]
    fn valid_workflow_passes() {
        let report =
            workflow("on: push\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n");
        assert!(report.ok() && report.warnings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_without_jobs_or_invalid_yaml_fails() {
        assert!(workflow("on: push\n").errors[0].contains("`jobs:`"));
        assert!(workflow("on: [push\n").errors[0].contains("Invalid YAML"));
    }

    #[test]
    fn unpinned_actions_are_warned_but_local_ones_are_not() {
        let report = workflow(
            "on: push\njobs:\n  a:\n    steps:\n      - uses: actions/checkout\n      - uses: ./local\n",
        );
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("actions/checkout"));
    }

    #[test]
    fn workspace_manifest_needs_shared_package_fields() {
        assert!(manifest("[workspace]\n").is_err());
        assert!(manifest("[workspace.package]\nversion = \"1\"\nlicense = \"MIT\"\n").is_ok());
    }
}
