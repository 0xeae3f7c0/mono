# Lints the repository's GitHub Actions workflows: actionlint for correctness
# (with ShellCheck for run: scripts) and zizmor for security issues such as
# over-broad token permissions and template injection.
# Run `commit-actions-lint` from the repository root.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  actionlint,
  shellcheck,
  pkg-config,
  aws-lc,
  rust-jemalloc-sys,
}:

rustPlatform.buildRustPackage rec {
  pname = "zizmor";
  version = "1.32.0";

  src = fetchFromGitHub {
    owner = "zizmorcore";
    repo = "zizmor";
    rev = "v${version}";
    hash = "sha256-wBBMF6FNwzlJiVag5tE64YnDf5Se+J5kJ/beVfg9H/Q=";
  };

  # zizmor links its TLS stack against the system AWS-LC, like the nixpkgs package.
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [
    aws-lc
    rust-jemalloc-sys
  ];

  cargoHash = "sha256-b7tcTvECEFb1fBVSo6y+NL4YognKuOBjAVMIyrCXZGQ=";
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-actions-lint <<EOF
    #!/bin/sh
    # actionlint (with ShellCheck) then zizmor, over .github/workflows.
    set -eu
    PATH="${lib.makeBinPath [ shellcheck ]}:\$PATH" ${lib.getExe actionlint} -color
    exec $out/bin/zizmor --offline "\''${1:-.github/workflows}"
    EOF
    chmod +x $out/bin/commit-actions-lint
  '';

  meta = {
    description = "zizmor and actionlint, as commit-actions-lint";
    homepage = "https://github.com/zizmorcore/zizmor";
    license = lib.licenses.mit;
    mainProgram = "commit-actions-lint";
  };
}
