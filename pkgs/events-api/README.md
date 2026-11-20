# events-api

Activity event API over SQLite.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **store**).
Built with Rust, axum, sqlx.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-events-api.yml`](../../.github/workflows/pkg-events-api.yml), which runs when files in this folder change.
