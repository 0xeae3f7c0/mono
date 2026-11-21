# Scans the repository's working tree and git history for leaked secrets
# (API keys, tokens, private keys) with Gitleaks.
# Run `commit-secret-scan [dir]` (defaults to the current directory).
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "gitleaks";
  version = "8.30.1";

  src = fetchFromGitHub {
    owner = "gitleaks";
    repo = "gitleaks";
    rev = "v${version}";
    hash = "sha256-l+T0nmsZcFV+OCZZzW6emtGJKsPwAZSPeOi23LkBHRM=";
  };

  vendorHash = "sha256-whJtl34dNltH/dk9qWSThcCYXC0x9PzbAUOO97Int+k=";

  ldflags = [
    "-s"
    "-w"
    "-X=github.com/zricethezav/gitleaks/v8/version.Version=${version}"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-secret-scan <<EOF
    #!/bin/sh
    # Scan the git history and working tree of the given directory.
    exec $out/bin/gitleaks git --redact --no-banner "\''${1:-.}"
    EOF
    chmod +x $out/bin/commit-secret-scan
  '';

  meta = {
    description = "Gitleaks plus commit-secret-scan, a whole-repo secret scan";
    homepage = "https://github.com/gitleaks/gitleaks";
    license = lib.licenses.mit;
    mainProgram = "commit-secret-scan";
  };
}
