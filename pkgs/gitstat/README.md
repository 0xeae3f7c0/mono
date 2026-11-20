# gitstat

Commits per author straight from a clone.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **collect**).
Built with Rust, gitoxide.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-gitstat.yml`](../../.github/workflows/pkg-gitstat.yml), which runs when files in this folder change.
