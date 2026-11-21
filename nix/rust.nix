# Rust packages built with crane, using the toolchain from rust-toolchain.toml.
# Each project in pkgs/ is its own Cargo workspace, so each is built from its
# own directory and Cargo.lock. Tests run in the per-project CI workflows.
{ pkgs, crane }:
let
  inherit (pkgs) lib;

  toolchain = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
  craneLib = (crane.mkLib pkgs).overrideToolchain (_: toolchain);

  build =
    {
      src,
      pname,
      extraArgs ? { },
    }:
    craneLib.buildPackage (
      {
        inherit pname;
        src = craneLib.cleanCargoSource src;
        strictDeps = true;
        doCheck = false;
      }
      // extraArgs
    );

  # The root workspace's `commit` CLI.
  commit = build {
    src = ../.;
    pname = "commit";
    extraArgs.cargoExtraArgs = "--locked -p commit";
  };

  # Services from pkgs/ that build without extra system libraries.
  services = lib.genAttrs [ "events-api" "badges" "shortlinks" "activity-rpc" ] (
    name:
    build {
      src = ../pkgs + "/${name}";
      pname = name;
      extraArgs.cargoExtraArgs = "--locked";
    }
  );

  # Container images for the services deployed by deploy/helm/commit.
  ports = {
    events-api = "3000";
    badges = "8081";
  };
  images = lib.optionalAttrs pkgs.stdenv.isLinux (
    lib.mapAttrs' (
      name: port:
      lib.nameValuePair "${name}-image" (
        pkgs.dockerTools.buildLayeredImage {
          inherit name;
          tag = "latest";
          contents = [ pkgs.cacert ];
          config = {
            Cmd = [ "${services.${name}}/bin/${name}" ];
            Env = [ "ADDR=0.0.0.0:${port}" ];
            ExposedPorts."${port}/tcp" = { };
          };
        }
      )
    ) ports
  );
in
{
  packages = {
    inherit commit;
  }
  // services
  // images;
}
