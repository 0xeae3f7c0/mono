# Checks and formats the monorepo's YAML (workflows, Dependabot, Helm values,
# the portal catalog) with yamlfmt. Helm templates are skipped: they are Go
# templates, not plain YAML.
# Run `commit-yaml-format [--fix]` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "yamlfmt";
  version = "0.21.0";

  src = fetchFromGitHub {
    owner = "google";
    repo = "yamlfmt";
    rev = "v${version}";
    hash = "sha256-AoMLm+YOxGiFcZZ/g617HNVuL/RIQueGJwgqTsFvM8I=";
  };

  vendorHash = "sha256-UTfbfAjmWKrHr/YxaWuaFswF3EnqcE8Otnz/sPoYT5w=";

  subPackages = [ "cmd/yamlfmt" ];
  ldflags = [
    "-s"
    "-w"
    "-X main.version=${version}"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-yaml-format <<EOF
    #!/bin/sh
    # Lint every tracked YAML file outside Helm templates; --fix rewrites them.
    set -eu
    files=\$(git ls-files '*.yml' '*.yaml' ':!:**/templates/**')
    if [ "\''${1:-}" = --fix ]; then
      exec $out/bin/yamlfmt \$files
    fi
    exec $out/bin/yamlfmt -lint \$files
    EOF
    chmod +x $out/bin/commit-yaml-format
  '';

  meta = {
    description = "yamlfmt plus commit-yaml-format, a whole-repo YAML format check";
    homepage = "https://github.com/google/yamlfmt";
    license = lib.licenses.asl20;
    mainProgram = "commit-yaml-format";
  };
}
