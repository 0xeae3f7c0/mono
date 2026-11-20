//! Counts commits per author on HEAD using gitoxide (no git binary needed).
//! Feeds the per-author numbers shown in `dashboard` and the weekly report.
//!
//!   gitstat [repo-path]

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

/// Commit counts per author name, busiest first (ties by name).
fn authors(repo_path: &Path) -> Result<Vec<(String, usize)>> {
    let repo = gix::discover(repo_path)?;
    let head = repo.head_commit()?;
    let mut counts = BTreeMap::<String, usize>::new();
    for info in head.ancestors().all()? {
        let commit = info?.object()?;
        *counts.entry(commit.author()?.name.to_string()).or_default() += 1;
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Ok(sorted)
}

fn main() -> Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    for (name, count) in authors(Path::new(&path))? {
        println!("{count:>6}  {name}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn counts_commits_by_author() {
        let dir = std::env::temp_dir().join(format!("gitstat-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str], name: &str| {
            let ok = Command::new("git")
                .args([
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    &format!("user.name={name}"),
                    "-c",
                    "user.email=t@example.com",
                ])
                .args(args)
                .current_dir(&dir)
                .status()
                .unwrap()
                .success();
            assert!(ok, "git {args:?} failed");
        };
        git(&["init", "-q"], "x");
        for name in ["alice", "bob", "alice"] {
            git(&["commit", "-q", "--allow-empty", "-m", "c"], name);
        }
        assert_eq!(
            authors(&dir).unwrap(),
            [("alice".into(), 2), ("bob".into(), 1)]
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
