# unused-deps

Finds dependencies declared in Cargo.toml that the code never uses, across
every Rust project in the monorepo, with cargo-machete.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned cargo-machete release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#unused-deps
```

The wrapper is `commit-unused-deps`; the package also provides the plain `cargo-machete` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-unused-deps.yml`](../../.github/workflows/pkg-unused-deps.yml), which builds it when this folder or the flake changes.
