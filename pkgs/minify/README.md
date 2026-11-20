# minify

Minifies the widget bundle.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **build**).
Built with Rust, SWC.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-minify.yml`](../../.github/workflows/pkg-minify.yml), which runs when files in this folder change.
