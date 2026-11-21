# sbom

Writes a CycloneDX software bill of materials (SBOM) for the root workspace and every project in `pkgs/`, using [Syft](https://github.com/anchore/syft).

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with Nix from a pinned Syft release; `nix-update` opens a pull request when a
new release is out.

## Use

```bash
nix run .#sbom            # writes sbom/<project>.cdx.json
nix run .#sbom -- out-dir   # choose the output directory
```

CI: [`.github/workflows/pkg-sbom.yml`](../../.github/workflows/pkg-sbom.yml), which builds it when this folder or the flake changes.
