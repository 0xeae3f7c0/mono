# summarize

Group-by summaries of exports.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **analyze**).
Built with Rust, polars.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-summarize.yml`](../../.github/workflows/pkg-summarize.yml), which runs when files in this folder change.
