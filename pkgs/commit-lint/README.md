# commit-lint

Checks that commit messages follow Conventional Commits (the format
release-please and the PR title check rely on) with committed.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned committed release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#commit-lint
```

The wrapper is `commit-commit-lint`; the package also provides the plain `committed` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-commit-lint.yml`](../../.github/workflows/pkg-commit-lint.yml), which builds it when this folder or the flake changes.
