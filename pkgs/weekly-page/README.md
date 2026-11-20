# weekly-page

Server-rendered "this week" page.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **report**).
Built with Rust, Leptos.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-weekly-page.yml`](../../.github/workflows/pkg-weekly-page.yml), which runs when files in this folder change.
