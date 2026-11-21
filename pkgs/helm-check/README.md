# helm-check

Validates the deploy chart in `deploy/helm/commit`: `helm lint`, then renders it and checks every manifest against the Kubernetes schemas with [kubeconform](https://github.com/yannh/kubeconform).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned kubeconform release; `nix-update` opens a pull request when a
new release is out.

## Use

```bash
nix run .#helm-check               # check deploy/helm/commit
nix run .#helm-check -- path/to/chart
```

CI: [`.github/workflows/pkg-helm-check.yml`](../../.github/workflows/pkg-helm-check.yml), which builds it when this folder or the flake changes.
