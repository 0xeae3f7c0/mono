# archive-stats

Reports archive bucket usage.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **report**).
Built with Go, AWS SDK v2.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-archive-stats.yml`](../../.github/workflows/pkg-archive-stats.yml), which runs when files in this folder change.
