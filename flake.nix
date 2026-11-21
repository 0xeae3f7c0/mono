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
    # `nix fmt` through treefmt, and a formatting check in `nix flake check`.
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Git hooks installed by the dev shell, also run as a flake check.
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # RustSec advisories for the cargo-audit flake check.
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    # Python projects in pkgs/ build from their uv.lock with uv2nix (nix/python.nix).
    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      treefmt-nix,
      git-hooks,
      advisory-db,
      pyproject-nix,
      uv2nix,
      pyproject-build-systems,
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
        python = import ./nix/python.nix {
          inherit
            pkgs
            pyproject-nix
            uv2nix
            pyproject-build-systems
            ;
        };
        # Nix projects in pkgs/ (one directory per package, see nix/pkgs.nix).
        nixProjects = import ./nix/pkgs.nix {
          inherit pkgs;
          extra = {
            commitPackages = rust.packages;
            devTools = devPackages.tools;
            nixosModule = self.nixosModules.default;
          };
        };
        treefmt = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs.nixfmt.enable = true;
        };
        preCommit = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            nixfmt.enable = true;
            check-merge-conflicts.enable = true;
          };
        };
      in
      {
        # `nix build .#<name>`: the CLI, the Rust services, the Python projects, (on Linux) their
        # container images, and every Nix project in pkgs/.
        packages =
          rust.packages
          // python
          // nixProjects
          // {
            default = rust.packages.commit;
          };

        # `nix develop` (or direnv with .envrc): every toolchain the monorepo uses.
        devShells.default = pkgs.mkShell {
          packages = devPackages.tools;
          buildInputs = devPackages.nativeLibs;
          shellHook = preCommit.shellHook + ''
            corepack enable --install-directory "$PWD/.direnv/bin" 2>/dev/null || true
            export PATH="$PWD/.direnv/bin:$PATH"
          '';
        };

        # `nix fmt` (treefmt running nixfmt)
        formatter = treefmt.config.build.wrapper;

        # `nix flake check`: formatting, the git hooks, and RustSec advisories
        # against the root Cargo.lock.
        checks = {
          formatting = treefmt.config.build.check self;
          pre-commit = preCommit;
          cargo-audit = (crane.mkLib pkgs).cargoAudit {
            src = ./.;
            inherit advisory-db;
          };
        };
      }
    )
    // {
      # NixOS module for running the platform services (pkgs/nixos-module).
      nixosModules.default = import ./pkgs/nixos-module/module.nix { inherit self; };
    };
}
