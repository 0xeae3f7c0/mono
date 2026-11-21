# cargo-deny

Checks every Cargo workspace in the monorepo for security advisories, banned
or duplicate crates and untrusted sources with cargo-deny.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned cargo-deny release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#cargo-deny
```

The wrapper is `commit-cargo-deny`; the package also provides the plain `cargo-deny` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-cargo-deny.yml`](../../.github/workflows/pkg-cargo-deny.yml), which builds it when this folder or the flake changes.
