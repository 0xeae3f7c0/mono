# runner-fleet

CI runner pods on Kubernetes.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Go, client-go.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-runner-fleet.yml`](../../.github/workflows/pkg-runner-fleet.yml), which runs when files in this folder change.
