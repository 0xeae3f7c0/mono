{
  description = "commit: self-hosted engineering-activity platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # Rust toolchain matching rust-toolchain.toml, for reproducible package builds.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Builds Cargo projects from their Cargo.lock with cached dependency layers.
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        devPackages = import ./nix/packages.nix { inherit pkgs; };
        rust = import ./nix/rust.nix { inherit pkgs crane; };
        # Nix projects in pkgs/ (one directory per package, see nix/pkgs.nix).
        nixProjects = import ./nix/pkgs.nix {
          inherit pkgs;
          extra = {
            commitPackages = rust.packages;
            devTools = devPackages.tools;
          };
        };
      in
      {
        # `nix build .#<name>`: the CLI, the Rust services, (on Linux) their
        # container images, and every Nix project in pkgs/.
        packages =
          rust.packages
          // nixProjects
          // {
            default = rust.packages.commit;
          };

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
