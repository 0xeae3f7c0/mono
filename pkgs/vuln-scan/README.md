# vuln-scan

Scans every lockfile in the monorepo (Cargo, Go, npm, Yarn) for known vulnerabilities
with [OSV-Scanner](https://github.com/google/osv-scanner), independently of Dependabot.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned OSV-Scanner release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#vuln-scan               # scan the whole repository
nix run .#vuln-scan -- pkgs/docs  # scan one project
```

The package also provides the plain `osv-scanner` binary.

CI: [`.github/workflows/pkg-vuln-scan.yml`](../../.github/workflows/pkg-vuln-scan.yml), which builds it when this folder or the flake changes.
