# Self-hosted GitHub Actions runner image with every monorepo toolchain, so CI
# jobs skip per-job installs. The entrypoint registers an ephemeral runner (one
# job, then exit) from RUNNER_URL and RUNNER_TOKEN; see README.md.
# Built with `nix build .#ci-runner-image` (a docker-archive tarball).
{
  lib,
  dockerTools,
  buildEnv,
  writeShellApplication,
  devTools,
  github-runner,
  bashInteractive,
  coreutils,
  findutils,
  gnugrep,
  gnused,
  gnutar,
  gzip,
  zstd,
  which,
  procps,
  inetutils,
}:

let
  entrypoint = writeShellApplication {
    name = "runner-entrypoint";
    runtimeInputs = [
      github-runner
      inetutils
    ];
    text = ''
      : "''${RUNNER_URL:?set RUNNER_URL, e.g. https://github.com/owner/repo}"
      : "''${RUNNER_TOKEN:?set RUNNER_TOKEN to a runner registration token}"
      mkdir -p "$RUNNER_ROOT" /runner/_work
      config.sh --unattended --ephemeral --disableupdate \
        --url "$RUNNER_URL" --token "$RUNNER_TOKEN" \
        --name "''${RUNNER_NAME:-$(hostname)}" \
        --labels "''${RUNNER_LABELS:-self-hosted,nix,commit}" \
        --work /runner/_work
      exec run.sh
    '';
  };

  env = buildEnv {
    name = "ci-runner-env";
    paths = devTools ++ [
      github-runner
      entrypoint
      bashInteractive
      coreutils
      findutils
      gnugrep
      gnused
      gnutar
      gzip
      zstd
      which
      procps
    ];
  };
in
(dockerTools.buildLayeredImage {
  name = "ci-runner";
  tag = "latest";
  contents = [
    env
    dockerTools.usrBinEnv
    dockerTools.binSh
    dockerTools.caCertificates
    dockerTools.fakeNss
  ];
  extraCommands = ''
    mkdir -p tmp runner
    chmod 1777 tmp
  '';
  config = {
    Entrypoint = [ (lib.getExe entrypoint) ];
    WorkingDir = "/runner";
    Env = [
      "PATH=${env}/bin:/usr/bin:/bin"
      "RUNNER_ROOT=/runner/state"
      "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
      "LANG=C.UTF-8"
      "HOME=/runner"
    ];
  };
  maxLayers = 100;
}).overrideAttrs
  (old: {
    meta = (old.meta or { }) // {
      description = "Ephemeral self-hosted GitHub Actions runner with every monorepo toolchain";
      platforms = [ "x86_64-linux" ];
    };
  })
