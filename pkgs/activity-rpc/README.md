# activity-rpc

Internal "how active is repo X" gRPC service.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **serve**).
Built with Rust, tonic.

## Develop

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --help   # or see src/main.rs for usage
```

CI: [`.github/workflows/pkg-activity-rpc.yml`](../../.github/workflows/pkg-activity-rpc.yml), which runs when files in this folder change.
