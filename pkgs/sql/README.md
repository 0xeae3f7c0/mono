# sql

Ad-hoc SQL over exports.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **analyze**).
Built with Rust, DataFusion.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-sql.yml`](../../.github/workflows/pkg-sql.yml), which runs when files in this folder change.
