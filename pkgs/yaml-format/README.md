# yaml-format

Checks and formats the monorepo's YAML (workflows, Dependabot, Helm values,
the portal catalog) with yamlfmt. Helm templates are skipped: they are Go
templates, not plain YAML.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned yamlfmt release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#yaml-format
```

The wrapper is `commit-yaml-format`; the package also provides the plain `yamlfmt` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-yaml-format.yml`](../../.github/workflows/pkg-yaml-format.yml), which builds it when this folder or the flake changes.
