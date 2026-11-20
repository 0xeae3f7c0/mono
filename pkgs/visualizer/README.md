# visualizer

"Office TV" view of author activity.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **show**).
Built with Rust, bevy.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-visualizer.yml`](../../.github/workflows/pkg-visualizer.yml), which runs when files in this folder change.
