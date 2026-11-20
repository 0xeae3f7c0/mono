# webhooks

Verifies GitHub webhook signatures, forwards events to `events-api`, Prometheus metrics.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **collect**).
Built with Go, gin.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-webhooks.yml`](../../.github/workflows/pkg-webhooks.yml), which runs when files in this folder change.
