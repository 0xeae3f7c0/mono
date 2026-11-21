# shell-format

Checks the formatting of every tracked shell script with
[shfmt](https://github.com/mvdan/sh) (2-space indent, indented case branches).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned shfmt release; `nix-update` opens a pull request
when a new release is out.

## Use

```bash
nix run .#shell-format             # show a diff for unformatted scripts
nix run .#shell-format -- --fix    # format in place
```

The package also provides the plain `shfmt` binary.

CI: [`.github/workflows/pkg-shell-format.yml`](../../.github/workflows/pkg-shell-format.yml), which builds it when this folder or the flake changes.
