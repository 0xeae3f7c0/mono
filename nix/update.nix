# Non-flake entry point for nix-update: `nix-update -f nix/update.nix <name>`.
# Evaluating through the flake breaks when nix-update edits files mid-run, so
# this loads the same pkgs/ packages with the nixpkgs revision from flake.lock
# (or nixos-unstable before flake.lock exists). Only packages that pin their own
# upstream release are updated this way; monorepo-only arguments are stubs.
{ ... }:
let
  lockFile = ../flake.lock;
  nixpkgsSrc =
    if builtins.pathExists lockFile then
      let
        lock = builtins.fromJSON (builtins.readFile lockFile);
        locked = lock.nodes.${lock.nodes.root.inputs.nixpkgs}.locked;
      in
      fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/${locked.rev}.tar.gz";
        sha256 = locked.narHash;
      }
    else
      fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz";
  pkgs = import nixpkgsSrc { };
in
import ./pkgs.nix {
  inherit pkgs;
  extra = {
    commitPackages = { };
    devTools = [ ];
  };
}
