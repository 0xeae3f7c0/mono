# crawl

Release-note/changelog links for dependency PRs.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **collect**).
Built with Rust, reqwest, scraper.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-crawl.yml`](../../.github/workflows/pkg-crawl.yml), which runs when files in this folder change.
