# Checks the Helm chart (deploy/helm/commit) against Kubernetes best practices
# (resource limits, probes, non-root containers) with KubeLinter.
# Run `commit-kube-lint [chart-dir]` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "kube-linter";
  version = "0.8.3";

  src = fetchFromGitHub {
    owner = "stackrox";
    repo = "kube-linter";
    rev = "v${version}";
    hash = "sha256-JpabvLiYnDglf6rLr7HLGPafs9HHV/GZBErXr7CBQbo=";
  };

  vendorHash = "sha256-xG4RsgPOoCWFhMEMFyGKQB05O44Pm1jFo2YC8zal1Q0=";

  subPackages = [ "cmd/kube-linter" ];
  ldflags = [
    "-s"
    "-w"
    "-X golang.stackrox.io/kube-linter/internal/version.version=${version}"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-kube-lint <<EOF
    #!/bin/sh
    # Render and lint the given chart directory.
    exec $out/bin/kube-linter lint "\''${1:-deploy/helm/commit}"
    EOF
    chmod +x $out/bin/commit-kube-lint
  '';

  meta = {
    description = "KubeLinter plus commit-kube-lint, a Helm chart best-practice check";
    homepage = "https://github.com/stackrox/kube-linter";
    license = lib.licenses.asl20;
    mainProgram = "commit-kube-lint";
  };
}
