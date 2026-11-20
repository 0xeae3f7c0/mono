# pipelines

Weekly report workflow.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **report**).
Built with Go, Temporal.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-pipelines.yml`](../../.github/workflows/pkg-pipelines.yml), which runs when files in this folder change.
