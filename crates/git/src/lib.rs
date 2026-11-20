// commit CLI: repository validation (crates/)
//! Thin wrappers around the `git` executable.

use std::path::Path;
use std::process::Command;

/// Errors from running `git`.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("could not execute git: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("cannot compare with {0}; ensure the base branch is fetched")]
    UnknownBase(String),
}

/// Runs `git diff --check` and returns a warning if it reports problems
/// or the directory is not a Git repository.
pub fn whitespace_warning(path: &Path) -> Option<String> {
    match Command::new("git")
        .args(["diff", "--check"])
        .current_dir(path)
        .output()
    {
        Ok(out) if out.status.success() => None,
        Ok(out) if String::from_utf8_lossy(&out.stderr).contains("Not a git repository") => {
            Some("Not a Git repository; skipped git diff --check.".into())
        }
        Ok(out) => Some(format!(
            "git diff --check found whitespace errors:\n{}",
            String::from_utf8_lossy(&out.stdout).trim_end()
        )),
        Err(error) => Some(format!("Could not run git diff --check: {error}")),
    }
}

/// Files changed between `base` and `HEAD` (three-dot diff).
pub fn changed_files(path: &Path, base: &str) -> Result<Vec<String>, GitError> {
    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("{base}...HEAD")])
        .current_dir(path)
        .output()?;
    if !output.status.success() {
        return Err(GitError::UnknownBase(base.to_owned()));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_repository_is_reported() {
        let dir = tempfile::TempDir::new().unwrap();
        let warning = whitespace_warning(dir.path()).unwrap();
        assert!(warning.contains("Not a Git repository"));
    }

    #[test]
    fn changed_files_fails_without_repository() {
        let dir = tempfile::TempDir::new().unwrap();
        let error = changed_files(dir.path(), "origin/main").unwrap_err();
        assert!(matches!(error, GitError::UnknownBase(base) if base == "origin/main"));
    }
}
