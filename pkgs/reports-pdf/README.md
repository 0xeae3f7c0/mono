# reports-pdf

Renders the report to PDF.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **report**).
Built with Rust, Typst.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-reports-pdf.yml`](../../.github/workflows/pkg-reports-pdf.yml), which runs when files in this folder change.
