# Nix packages in pkgs/: every pkgs/<name>/default.nix is a package, built with
# callPackage. Arguments it asks for come from nixpkgs or from `extra` (the
# monorepo's own packages and tools). Packages that do not support the current
# system (for example Linux-only images) are left out instead of failing.
{ pkgs, extra }:
let
  inherit (pkgs) lib;
  callPackage = lib.callPackageWith (pkgs // extra);
  isNixProject =
    name: type: type == "directory" && builtins.pathExists (../pkgs + "/${name}/default.nix");
  all = lib.mapAttrs (name: _: callPackage (../pkgs + "/${name}") { }) (
    lib.filterAttrs isNixProject (builtins.readDir ../pkgs)
  );
in
lib.filterAttrs (_: drv: lib.meta.availableOn pkgs.stdenv.hostPlatform drv) all
