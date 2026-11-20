# event-store

Offline buffer for events, replayed later.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **store**).
Built with Go, GORM.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-event-store.yml`](../../.github/workflows/pkg-event-store.yml), which runs when files in this folder change.
