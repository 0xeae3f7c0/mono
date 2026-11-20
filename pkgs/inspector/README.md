# inspector

Native viewer for author exports.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **show**).
Built with Rust, iced.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-inspector.yml`](../../.github/workflows/pkg-inspector.yml), which runs when files in this folder change.
