# helm-render

Renders `deploy/helm/commit` for review.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Go, Helm.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-helm-render.yml`](../../.github/workflows/pkg-helm-render.yml), which runs when files in this folder change.
