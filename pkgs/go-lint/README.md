# go-lint

Runs [golangci-lint](https://github.com/golangci/golangci-lint) in every Go module
of the monorepo (go/ and pkgs/*), one module at a time.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned golangci-lint release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#go-lint   # lint every Go module
```

The package also provides the plain `golangci-lint` binary.

CI: [`.github/workflows/pkg-go-lint.yml`](../../.github/workflows/pkg-go-lint.yml), which builds it when this folder or the flake changes.
