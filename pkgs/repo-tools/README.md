# repo-tools

The repository's own scripts as commands with pinned dependencies (Node.js, jq, gh,
git, cargo, go), so they behave the same on every machine and in CI.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix; dependencies follow the flake's nixpkgs input.

## Use

Run from the repository root:

```bash
nix run .#repo-tools                          # commit-gen: regenerate generated files
nix shell .#repo-tools -c commit-gen --check  # verify they are current
nix shell .#repo-tools -c commit-deps         # unique dependency counts
nix shell .#repo-tools -c commit-labels       # print the label set
nix shell .#repo-tools -c commit-update-deps  # open dependency PRs (needs GH_TOKEN)
```

CI: [`.github/workflows/pkg-repo-tools.yml`](../../.github/workflows/pkg-repo-tools.yml), which builds it when this folder or the flake changes.
