# repo-registry

Which repositories are tracked.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **store**).
Built with Rust, SeaORM.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-repo-registry.yml`](../../.github/workflows/pkg-repo-registry.yml), which runs when files in this folder change.
