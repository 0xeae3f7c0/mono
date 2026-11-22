# repo-settings

The monorepo's GitHub settings as code, written in [OpenTofu](https://opentofu.org/)
(HCL) with the `integrations/github` provider: a ruleset on `main` that requires
the `Rust checks` and `Generated files` CI checks, so auto-merge waits for them.
Force-pushes and branch deletion stay allowed. Repository admins can bypass it.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
The first HCL project in the monorepo; Dependabot keeps the provider version current.

## Apply

Run by hand with a token that can administer the repository; CI never applies.
State stays local (`terraform.tfstate`, git-ignored).

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
