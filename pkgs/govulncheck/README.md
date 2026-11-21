# govulncheck

Reports known vulnerabilities that the monorepo's Go code actually calls,
module by module, with govulncheck (Go's own call-graph-aware scanner).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned govulncheck release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#govulncheck
```

The wrapper is `commit-govulncheck`; the package also provides the plain `govulncheck` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-govulncheck.yml`](../../.github/workflows/pkg-govulncheck.yml), which builds it when this folder or the flake changes.
