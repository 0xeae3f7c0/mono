# markdown-lint

Lints the monorepo's Markdown (READMEs, docs, changelog) for broken
structure and style with rumdl.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned rumdl release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#markdown-lint
```

The wrapper is `commit-markdown-lint`; the package also provides the plain `rumdl` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-markdown-lint.yml`](../../.github/workflows/pkg-markdown-lint.yml), which builds it when this folder or the flake changes.
