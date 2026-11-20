# shortlinks

Short links for shared reports.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **serve**).
Built with Rust, Rocket.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-shortlinks.yml`](../../.github/workflows/pkg-shortlinks.yml), which runs when files in this folder change.
