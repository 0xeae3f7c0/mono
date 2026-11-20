# plugin-host

Sandboxed custom metrics as WebAssembly plugins.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **analyze**).
Built with Rust, wasmtime.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-plugin-host.yml`](../../.github/workflows/pkg-plugin-host.yml), which runs when files in this folder change.
