# Reports known vulnerabilities that the monorepo's Go code actually calls,
# module by module, with govulncheck (Go's own call-graph-aware scanner).
# Run `commit-govulncheck` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
  go,
}:

buildGoModule rec {
  pname = "govulncheck";
  version = "1.10.0";

  src = fetchFromGitHub {
    owner = "golang";
    repo = "vuln";
    rev = "v${version}";
    hash = "sha256-bpf9bf5D9nMet3wikioU/fRchzE6R0bQ9ErO0FGKQ7Q=";
  };

  vendorHash = "sha256-tVHoUdJKPlSxYvUkKPAF9DLqFlc/IsdwVFtIJpZ9N7E=";

  subPackages = [ "cmd/govulncheck" ];
  ldflags = [
    "-s"
    "-w"
  ];
  doCheck = false;
  # The wrapper puts go on PATH, which buildGoModule otherwise forbids.
  allowGoReference = true;

  postInstall = ''
    cat > $out/bin/commit-govulncheck <<EOF
    #!/bin/sh
    # govulncheck loads packages with the go command.
    export PATH="${lib.makeBinPath [ go ]}:\$PATH"
    # Run govulncheck in each directory that has a go.mod.
    set -u
    status=0
    for mod in \$(git ls-files '*go.mod'); do
      dir=\$(dirname "\$mod")
      echo "== \$dir"
      (cd "\$dir" && $out/bin/govulncheck ./...) || status=1
    done
    exit \$status
    EOF
    chmod +x $out/bin/commit-govulncheck
  '';

  meta = {
    description = "govulncheck plus commit-govulncheck, a scan of every Go module";
    homepage = "https://github.com/golang/vuln";
    license = lib.licenses.bsd3;
    mainProgram = "commit-govulncheck";
  };
}
