# archive

Uploads generated reports to S3.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **report**).
Built with Rust, AWS SDK.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-archive.yml`](../../.github/workflows/pkg-archive.yml), which runs when files in this folder change.
