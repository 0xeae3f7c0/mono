# toml-lint

Lints and checks the formatting of every tracked TOML file (Cargo.toml manifests,
rust-toolchain.toml, tool configs) with [Taplo](https://github.com/tamasfe/taplo).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned Taplo release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#toml-lint             # lint and check formatting
nix run .#toml-lint -- --fix    # format in place
```

The package also provides the plain `taplo` binary.

CI: [`.github/workflows/pkg-toml-lint.yml`](../../.github/workflows/pkg-toml-lint.yml), which builds it when this folder or the flake changes.
