# repo-settings

The monorepo's GitHub settings as code, written in [OpenTofu](https://opentofu.org/)
(HCL) with the `integrations/github` provider: a ruleset on `main` that requires
the `Rust checks` and `Generated files` CI checks, so auto-merge waits for them.
Force-pushes and branch deletion stay allowed. Repository admins can bypass it.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
The first HCL project in the monorepo; Renovate keeps the provider version current.

## Apply

[`.github/workflows/repo-settings-apply.yml`](../../.github/workflows/repo-settings-apply.yml)
applies it with `OWNER_TOKEN` on every push to `main` that changes this folder,
daily (to undo changes made in the UI), and on demand. It keeps no state: each
run imports the existing `main` ruleset by name, then applies.

By hand, with a token that can administer the repository:

```bash
export GITHUB_TOKEN=...            # admin access to the repository
tofu init
tofu plan                          # review the ruleset change
tofu apply
```

If `main` already has a matching ruleset, import it first:
`tofu import github_repository_ruleset.main mono:<ruleset id>`.

## Develop

```bash
tofu fmt -check -recursive
tofu init -backend=false && tofu validate
nix run .#tf-lint                   # TFLint
```

CI: [`.github/workflows/pkg-repo-settings.yml`](../../.github/workflows/pkg-repo-settings.yml), which runs when files in this folder change.
