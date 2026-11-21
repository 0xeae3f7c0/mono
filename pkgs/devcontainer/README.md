# devcontainer

Development container image for GitHub Codespaces and VS Code dev containers, with
every toolchain from the Nix dev shell already installed (Rust, Go, Node.js, just,
lefthook, treefmt, gh, jq, shellcheck). Opening a Codespace gives a ready
environment: no installing Rust or Yarn by hand.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix (`dockerTools`); tools follow the flake's nixpkgs input. It also
provides the glibc loader path and C++ runtime that the VS Code server needs.

## Use

```bash
nix build .#devcontainer        # result: docker-archive tarball
docker load < result            # loads devcontainer:latest
```

Once published to GitHub Container Registry, Codespaces uses it through
`.devcontainer/devcontainer.json`.

CI: [`.github/workflows/pkg-devcontainer.yml`](../../.github/workflows/pkg-devcontainer.yml), which builds it when this folder or the flake changes.
