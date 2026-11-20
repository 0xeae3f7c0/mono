// archive: Uploads generated reports to S3 (commit platform, pkgs/archive)
//! Archives generated reports (a local directory) to an S3 bucket under a key
//! prefix, e.g. `reports/2026-W47/`. `archive-stats` reports bucket usage.
//!
//!   s3sync ./site my-bucket --prefix releases/v1 --dry-run

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;
use clap::Parser;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Local directory to upload
    source: PathBuf,
    /// Target bucket
    bucket: String,
    /// Key prefix inside the bucket
    #[arg(long, default_value = "")]
    prefix: String,
    /// Print the plan without uploading
    #[arg(long)]
    dry_run: bool,
}

/// Files under `root` paired with their S3 keys, sorted by key.
fn plan(root: &Path, prefix: &str) -> Result<Vec<(PathBuf, String)>> {
    let prefix = prefix.trim_matches('/');
    let mut items = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry.context("walking source directory")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry.path().strip_prefix(root)?;
        let relative = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let key = if prefix.is_empty() {
            relative
        } else {
            format!("{prefix}/{relative}")
        };
        items.push((entry.into_path(), key));
    }
    items.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(items)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let items = plan(&args.source, &args.prefix)?;
    if args.dry_run {
        for (path, key) in &items {
            println!("{} -> s3://{}/{key}", path.display(), args.bucket);
        }
        return Ok(());
    }

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    for (path, key) in &items {
        let body = ByteStream::from_path(path)
            .await
            .with_context(|| format!("reading {}", path.display()))?;
        client
            .put_object()
            .bucket(&args.bucket)
            .key(key)
            .body(body)
            .send()
            .await
            .with_context(|| format!("uploading {key}"))?;
        println!("uploaded s3://{}/{key}", args.bucket);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_use_prefix_and_forward_slashes() {
        let dir = tempdir();
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::write(dir.join("a/b/c.txt"), "x").unwrap();
        std::fs::write(dir.join("root.txt"), "y").unwrap();

        let keys: Vec<_> = plan(&dir, "/releases/v1/")
            .unwrap()
            .into_iter()
            .map(|(_, k)| k)
            .collect();
        assert_eq!(keys, ["releases/v1/a/b/c.txt", "releases/v1/root.txt"]);

        let keys: Vec<_> = plan(&dir, "")
            .unwrap()
            .into_iter()
            .map(|(_, k)| k)
            .collect();
        assert_eq!(keys, ["a/b/c.txt", "root.txt"]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("s3sync-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
