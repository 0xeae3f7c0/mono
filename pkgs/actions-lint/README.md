# actions-lint

Lints the GitHub Actions workflows: [actionlint](https://github.com/rhysd/actionlint) (with ShellCheck) for correctness, then [zizmor](https://github.com/zizmorcore/zizmor) for security problems such as unpinned actions, over-broad token permissions and template injection.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned zizmor release; `nix-update` opens a pull request when a
new release is out.

## Use

```bash
nix run .#actions-lint   # lint .github/workflows
```

CI: [`.github/workflows/pkg-actions-lint.yml`](../../.github/workflows/pkg-actions-lint.yml), which builds it when this folder or the flake changes.
