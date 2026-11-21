# Checks and formats the monorepo's TOML files (Cargo.toml manifests,
# rust-toolchain.toml, tool configs) with Taplo.
# Run `commit-toml-lint [--fix]` from the repository root.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  openssl,
}:

rustPlatform.buildRustPackage rec {
  pname = "taplo";
  version = "0.10.0";

  src = fetchFromGitHub {
    owner = "tamasfe";
    repo = "taplo";
    rev = version;
    hash = "sha256-FW8OQ5TRUuQK8M2NDmp4c6p22jsHodxKqzOMrcdiqXU=";
  };

  cargoHash = "sha256-/xdiI+gOC5pnjH+qD/xM3dpJGjiqhCZ3EYKaeiN6InI=";
  cargoBuildFlags = [
    "-p"
    "taplo-cli"
  ];
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-toml-lint <<EOF
    #!/bin/sh
    # Lint every tracked TOML file; with --fix, format them in place instead.
    set -eu
    files=\$(git ls-files '*.toml')
    if [ "\''${1:-}" = --fix ]; then
      exec $out/bin/taplo format \$files
    fi
    $out/bin/taplo lint \$files
    exec $out/bin/taplo format --check \$files
    EOF
    chmod +x $out/bin/commit-toml-lint
  '';

  meta = {
    description = "Taplo plus commit-toml-lint, a whole-repo TOML check";
    homepage = "https://github.com/tamasfe/taplo";
    license = lib.licenses.mit;
    mainProgram = "commit-toml-lint";
  };
}
