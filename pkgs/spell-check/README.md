# spell-check

Finds common misspellings in source code and docs with
[typos](https://github.com/crate-ci/typos), which keeps false positives low
enough to run on a whole monorepo.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned typos release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#spell-check                        # report misspellings
nix run .#spell-check -- --write-changes     # fix them in place
nix run .#spell-check -- pkgs/docs           # check one project
```

The package also provides the plain `typos` binary.

CI: [`.github/workflows/pkg-spell-check.yml`](../../.github/workflows/pkg-spell-check.yml), which builds it when this folder or the flake changes.
