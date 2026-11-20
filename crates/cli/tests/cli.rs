use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn commit() -> Command {
    Command::cargo_bin("commit").expect("binary builds")
}

#[test]
fn check_passes_on_this_repository() {
    commit()
        .args([
            "--path",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../.."),
            "check",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository validation passed"));
}

#[test]
fn check_reports_missing_files_as_json() {
    let dir = TempDir::new().unwrap();
    let output = commit()
        .args(["--json", "--path"])
        .arg(dir.path())
        .arg("check")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let errors = report["errors"].as_array().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.as_str().unwrap().contains("Cargo.toml"))
    );
}

#[test]
fn manifest_without_package_is_rejected() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
    commit()
        .arg("--path")
        .arg(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "missing [package] or [workspace.package] table",
        ));
}

#[test]
fn workflow_without_trigger_is_rejected() {
    let dir = TempDir::new().unwrap();
    let workflows = dir.path().join(".github/workflows");
    std::fs::create_dir_all(&workflows).unwrap();
    std::fs::write(workflows.join("broken.yml"), "name: Broken\njobs: {}\n").unwrap();
    commit()
        .arg("--path")
        .arg(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no top-level `on:` trigger: .github/workflows/broken.yml",
        ));
}

#[test]
fn verbose_logs_to_stderr() {
    commit()
        .args([
            "-v",
            "--path",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../.."),
            "check",
        ])
        .env_remove("RUST_LOG")
        .assert()
        .success()
        .stderr(predicate::str::contains("validated repository"));
}
