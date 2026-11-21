# Development container image for Codespaces and VS Code dev containers, with
# every toolchain from the dev shell (Rust, Go, Node.js, just, lefthook, ...).
# Includes the glibc loader path and C++ runtime that the VS Code server needs.
# Built with `nix build .#devcontainer` (a docker-archive tarball).
{
  lib,
  dockerTools,
  buildEnv,
  devTools,
  bashInteractive,
  coreutils,
  findutils,
  gnugrep,
  gnused,
  gnutar,
  gzip,
  less,
  which,
  procps,
  openssh,
  glibc,
  stdenv,
}:

let
  env = buildEnv {
    name = "devcontainer-env";
    paths = devTools ++ [
      bashInteractive
      coreutils
      findutils
      gnugrep
      gnused
      gnutar
      gzip
      less
      which
      procps
      openssh
    ];
  };
in
(dockerTools.buildLayeredImage {
  name = "devcontainer";
  tag = "latest";
  contents = [
    env
    dockerTools.usrBinEnv
    dockerTools.binSh
    dockerTools.caCertificates
    dockerTools.fakeNss
  ];
  # VS Code server ships glibc-linked binaries that expect an FHS loader path.
  extraCommands = ''
    mkdir -p tmp workspaces lib64 root
    chmod 1777 tmp
    ln -s ${glibc}/lib/ld-linux-x86-64.so.2 lib64/ld-linux-x86-64.so.2
  '';
  config = {
    Cmd = [ "${bashInteractive}/bin/bash" ];
    WorkingDir = "/workspaces";
    Env = [
      "PATH=${env}/bin:/usr/bin:/bin"
      "LD_LIBRARY_PATH=${
        lib.makeLibraryPath [
          stdenv.cc.cc.lib
          glibc
        ]
      }"
      "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
      "LANG=C.UTF-8"
      "HOME=/root"
    ];
  };
  maxLayers = 100;
}).overrideAttrs
  (old: {
    meta = (old.meta or { }) // {
      description = "Codespaces / dev container image with every monorepo toolchain";
      platforms = [ "x86_64-linux" ];
    };
  })
