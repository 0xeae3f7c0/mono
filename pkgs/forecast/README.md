# forecast

Commit-volume trend and projection.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **analyze**).
Built with Rust, candle.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-forecast.yml`](../../.github/workflows/pkg-forecast.yml), which runs when files in this folder change.
