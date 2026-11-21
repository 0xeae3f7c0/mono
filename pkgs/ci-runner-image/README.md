# ci-runner-image

Self-hosted GitHub Actions runner image with every monorepo toolchain installed
(Rust, Go, Node.js and the rest of the Nix dev shell), so jobs skip per-job tool
installs. Each container registers an **ephemeral** runner: it takes one job, then
exits, which keeps jobs isolated from each other.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix (`dockerTools`) around the nixpkgs `github-runner`; tools follow the
flake's nixpkgs input. [`runner-fleet`](../runner-fleet) reports these runners on
Kubernetes.

## Use

```bash
nix build .#ci-runner-image && docker load < result
docker run --rm \
  -e RUNNER_URL=https://github.com/0xeae3f7c0/mono \
  -e RUNNER_TOKEN=<registration token from Settings → Actions → Runners> \
  ci-runner:latest
```

Optional: `RUNNER_NAME` (default: hostname) and `RUNNER_LABELS`
(default: `self-hosted,nix,commit`). Treat the registration token as a secret.

CI: [`.github/workflows/pkg-ci-runner-image.yml`](../../.github/workflows/pkg-ci-runner-image.yml), which builds it when this folder or the flake changes.
