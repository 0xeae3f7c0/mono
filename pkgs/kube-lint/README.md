# kube-lint

Checks the Helm chart (deploy/helm/commit) against Kubernetes best practices
(resource limits, probes, non-root containers) with KubeLinter.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned kube-linter release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#kube-lint
```

The wrapper is `commit-kube-lint`; the package also provides the plain `kube-linter` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-kube-lint.yml`](../../.github/workflows/pkg-kube-lint.yml), which builds it when this folder or the flake changes.
