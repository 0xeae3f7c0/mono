# alerts-ui

Alert-rule editor with live preview.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **show**).
Built with Rust, Dioxus.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-alerts-ui.yml`](../../.github/workflows/pkg-alerts-ui.yml), which runs when files in this folder change.
