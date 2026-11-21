# Contributing

Thanks for helping improve `commit`. See [`ARCHITECTURE.md`](ARCHITECTURE.md) for how the
projects fit together and [`doc/README.md`](doc/README.md) for the layout, automation and
repository settings.

## Requirements

- Stable Rust toolchain
- Go (latest stable; `pkgs/*` modules need a newer Go than `go/go.mod`)
- Node.js 22+ (and `corepack enable` for `pkgs/portal`, which uses Yarn)
- Git

## Before opening a pull request

For the root tools:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- check
(cd go && gofmt -l . && go vet ./... && go test ./...)
(cd web/status-worker && npm test)
```

For a project in `pkgs/`, run what its CI runs (see its `README.md`):

```bash
cargo clippy --all-targets -- -D warnings && cargo test   # Rust
go vet ./... && go test ./...                              # Go
npm ci && npm run ci                                       # npm
```

If you add or remove a project, regenerate the files CI checks:

```bash
node scripts/gen-pkg-workflows.mjs
node scripts/gen-catalog.mjs
```

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) with a subject of 50
characters or fewer, for example `fix(webhooks): reject unsigned payloads`. Dependency
pull requests follow the same format automatically.

## Sign-off and license

Contributions are dual-licensed `MIT OR Apache-2.0` and certified with the
[Developer Certificate of Origin](DCO-1.1): add `-s` to `git commit` so the message
ends with `Signed-off-by: Your Name <you@example.com>`. See [`COPYRIGHT.md`](COPYRIGHT.md).

## Guidelines

Keep changes focused, include regression tests for behaviour changes, update
documentation when needed, and never commit credentials or private data.

Automated checks are evidence, not human approval. Do not add workflows that fabricate
contribution activity or manufacture reviews.
