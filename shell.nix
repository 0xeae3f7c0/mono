# Non-flake entry point: `nix-shell` gives the same tools as `nix develop`.
{
  pkgs ? import <nixpkgs> { },
}:
let
  devPackages = import ./nix/packages.nix { inherit pkgs; };
in
pkgs.mkShell {
  packages = devPackages.tools;
  buildInputs = devPackages.nativeLibs;
}
