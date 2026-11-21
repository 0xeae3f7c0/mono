{
  description = "commit: self-hosted engineering-activity platform (development environment)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        devPackages = import ./nix/packages.nix { inherit pkgs; };
      in
      {
        # `nix develop` (or direnv with .envrc): every toolchain the monorepo uses.
        devShells.default = pkgs.mkShell {
          packages = devPackages.tools;
          buildInputs = devPackages.nativeLibs;
          shellHook = ''
            corepack enable --install-directory "$PWD/.direnv/bin" 2>/dev/null || true
            export PATH="$PWD/.direnv/bin:$PATH"
          '';
        };

        # `nix fmt`
        formatter = pkgs.nixfmt;
      }
    );
}
