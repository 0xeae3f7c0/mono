# desktop

CI status desktop app.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **show**).
Built with Tauri.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-desktop.yml`](../../.github/workflows/pkg-desktop.yml), which runs when files in this folder change.
