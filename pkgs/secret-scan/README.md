# secret-scan

Scans the repository's working tree and git history for leaked secrets (API keys,
tokens, private keys) with [Gitleaks](https://github.com/gitleaks/gitleaks).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned Gitleaks release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#secret-scan               # scan the whole repository and its history
nix run .#secret-scan -- pkgs/docs  # scan one project
```

Findings are redacted in the output. The package also provides the plain `gitleaks` binary.

CI: [`.github/workflows/pkg-secret-scan.yml`](../../.github/workflows/pkg-secret-scan.yml), which builds it when this folder or the flake changes.
