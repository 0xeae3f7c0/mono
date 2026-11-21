# Scans every lockfile in the monorepo (Cargo, Go, npm, Yarn) for known
# vulnerabilities with OSV-Scanner, offline from Dependabot.
# Run `commit-vuln-scan [dir]` (defaults to the current directory).
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "osv-scanner";
  version = "2.7.1";

  src = fetchFromGitHub {
    owner = "google";
    repo = "osv-scanner";
    rev = "v${version}";
    hash = "sha256-eIfrC9g7Y93DsuzN39X6GZAtS4xsKFyE4eXxL56tn+g=";
  };

  vendorHash = "sha256-8fB4ktq0sgFmWJ64NyiMYJZ1Q9sPiLlHpyFmIuu25iA=";

  subPackages = [ "cmd/osv-scanner" ];
  ldflags = [
    "-s"
    "-w"
    "-X=github.com/google/osv-scanner/v2/internal/version.OSVVersion=${version}"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-vuln-scan <<EOF
    #!/bin/sh
    # Recursively scan every lockfile under the given directory.
    exec $out/bin/osv-scanner scan source --recursive "\''${1:-.}"
    EOF
    chmod +x $out/bin/commit-vuln-scan
  '';

  meta = {
    description = "OSV-Scanner plus commit-vuln-scan, a whole-repo lockfile scan";
    homepage = "https://github.com/google/osv-scanner";
    license = lib.licenses.asl20;
    mainProgram = "commit-vuln-scan";
  };
}
