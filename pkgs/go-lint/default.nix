# Lints every Go module in the monorepo (go/ and pkgs/*) with golangci-lint.
# Run `commit-go-lint` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "golangci-lint";
  version = "2.15.0";

  src = fetchFromGitHub {
    owner = "golangci";
    repo = "golangci-lint";
    rev = "v${version}";
    hash = "sha256-m9PoJHescRuT9DiKsb6LL+lQlc7RKAWDiRyf8pNMffA=";
  };

  vendorHash = "sha256-gWbhZc/BJwuiwyTMaR1ShEBI1lo0bGLCXsdj5n91baw=";

  subPackages = [ "cmd/golangci-lint" ];
  ldflags = [
    "-s"
    "-w"
    "-X main.version=${version}"
    "-X main.commit=v${version}"
    "-X main.date=1970-01-01T00:00:00Z"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-go-lint <<EOF
    #!/bin/sh
    # Run golangci-lint in each directory that has a go.mod.
    set -u
    status=0
    for mod in \$(git ls-files '*go.mod'); do
      dir=\$(dirname "\$mod")
      echo "== \$dir"
      (cd "\$dir" && $out/bin/golangci-lint run ./...) || status=1
    done
    exit \$status
    EOF
    chmod +x $out/bin/commit-go-lint
  '';

  meta = {
    description = "golangci-lint plus commit-go-lint, a lint run over every Go module";
    homepage = "https://github.com/golangci/golangci-lint";
    license = lib.licenses.gpl3Only;
    mainProgram = "commit-go-lint";
  };
}
