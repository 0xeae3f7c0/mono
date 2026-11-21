# tf-lint

Lints the monorepo's OpenTofu / Terraform configurations (`pkgs/repo-settings`)
for errors, deprecated syntax and unused declarations with
[TFLint](https://github.com/terraform-linters/tflint).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned TFLint release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#tf-lint
```

The wrapper is `commit-tf-lint`; the package also provides the plain `tflint` binary.
`.github/workflows/lint.yml` runs it on pull requests (report only).

CI: [`.github/workflows/pkg-tf-lint.yml`](../../.github/workflows/pkg-tf-lint.yml), which builds it when this folder or the flake changes.
