# services

Running platform containers on a Docker host.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Go, moby client.

## Develop

```bash
go test ./...
go vet ./...
go run .
```

CI: [`.github/workflows/pkg-services.yml`](../../.github/workflows/pkg-services.yml), which runs when files in this folder change.
