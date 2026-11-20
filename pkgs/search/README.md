# search

Full-text search over docs and notes.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **analyze**).
Built with Rust, tantivy.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-search.yml`](../../.github/workflows/pkg-search.yml), which runs when files in this folder change.
