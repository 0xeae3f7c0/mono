# heatmap

Commit activity heatmap: commits per weekday and hour of the day from a
repository's git history, as a terminal grid or JSON.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **analyze**).
Built with Python and [uv](https://docs.astral.sh/uv/); the first Python project in the monorepo.

## Use

```bash
nix run .#heatmap                          # this repository
nix run .#heatmap -- ../other --since "90 days ago"
nix run .#heatmap -- --json                # {"weekdays": [...], "grid": [[...], ...]}
```

## Develop

```bash
uv sync
uv run pytest
uv run ruff check && uv run ruff format --check
```

CI: [`.github/workflows/pkg-heatmap.yml`](../../.github/workflows/pkg-heatmap.yml), which runs when files in this folder change.
