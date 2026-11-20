# Contributing

Thanks for helping improve `commit`.

## Requirements

- Stable Rust toolchain
- Go (see `go/go.mod`)
- Node.js 22+
- Git

## Before opening a pull request

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- check
(cd go && gofmt -l . && go vet ./... && go test ./...)
(cd web/status-worker && npm test)
```

Keep changes focused, include regression tests for behaviour changes, update documentation when needed, and never commit credentials or private data.

Automated checks are evidence, not human approval. Do not add workflows that fabricate contribution activity or manufacture reviews.
