<!-- repository tests guide -->
# Tests

- Rust unit tests live next to each crate in `crates/*/src`.
- CLI integration tests live in `crates/cli/tests/cli.rs`.
- Go tests live beside the code in `go/internal/**`.
- Worker tests live in `web/status-worker/test`.

```bash
cargo test --workspace
(cd go && go test ./...)
(cd web/status-worker && npm test)
```

Add regression tests whenever command behaviour or repository validation changes.
