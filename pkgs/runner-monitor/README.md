# runner-monitor

Terminal monitor for CI runner hosts.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **show**).
Built with Rust, ratatui.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-runner-monitor.yml`](../../.github/workflows/pkg-runner-monitor.yml), which runs when files in this folder change.
