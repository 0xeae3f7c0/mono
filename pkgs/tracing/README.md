# tracing

Reference OpenTelemetry setup for Go services.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Go, OpenTelemetry.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-tracing.yml`](../../.github/workflows/pkg-tracing.yml), which runs when files in this folder change.
