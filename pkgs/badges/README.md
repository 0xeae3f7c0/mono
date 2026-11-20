# badges

README badges (SVG).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **serve**).
Built with Rust, actix-web.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-badges.yml`](../../.github/workflows/pkg-badges.yml), which runs when files in this folder change.
