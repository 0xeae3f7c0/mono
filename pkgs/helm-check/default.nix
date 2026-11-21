# Validates the deploy chart (deploy/helm/commit): `helm lint`, then renders it
# and checks every manifest against the Kubernetes schemas with kubeconform.
# Run `commit-helm-check [chart-dir]` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
  kubernetes-helm,
}:

buildGoModule rec {
  pname = "kubeconform";
  version = "0.8.0";

  src = fetchFromGitHub {
    owner = "yannh";
    repo = "kubeconform";
    rev = "v${version}";
    hash = "sha256-HcYKmit67ZGABDfpr281FiStMZmM/vkkj9wxXH5H9zc=";
  };

  vendorHash = null;

  subPackages = [ "cmd/kubeconform" ];
  ldflags = [
    "-s"
    "-w"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-helm-check <<EOF
    #!/bin/sh
    # helm lint, then render and validate against the Kubernetes schemas.
    set -eu
    chart="\''${1:-deploy/helm/commit}"
    ${lib.getExe kubernetes-helm} lint "\$chart"
    ${lib.getExe kubernetes-helm} template commit "\$chart" | $out/bin/kubeconform -strict -summary -
    EOF
    chmod +x $out/bin/commit-helm-check
  '';

  meta = {
    description = "kubeconform plus commit-helm-check for the deploy chart";
    homepage = "https://github.com/yannh/kubeconform";
    license = lib.licenses.asl20;
    mainProgram = "commit-helm-check";
  };
}
