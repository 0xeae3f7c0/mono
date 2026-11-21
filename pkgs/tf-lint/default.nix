# Lints the monorepo's OpenTofu / Terraform configurations (pkgs/repo-settings)
# for errors, deprecated syntax and unused declarations with TFLint.
# Run `commit-tf-lint` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "tflint";
  version = "0.66.0";

  src = fetchFromGitHub {
    owner = "terraform-linters";
    repo = "tflint";
    rev = "v${version}";
    hash = "sha256-vQFqTgLFIkLkCrobcpq5bsK2cAHzejZYiJOYSf0kOvw=";
  };

  vendorHash = "sha256-tkazDw40Q/G1jAwyJhkrxChOmyXZyHzcXnHGIiMkPI0=";

  subPackages = [ "." ];
  ldflags = [
    "-s"
    "-w"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-tf-lint <<EOF
    #!/bin/sh
    # Run TFLint in each directory that has .tf files.
    set -u
    status=0
    for dir in \$(git ls-files '*.tf' | xargs -r -n1 dirname | sort -u); do
      echo "== \$dir"
      $out/bin/tflint --chdir="\$dir" || status=1
    done
    exit \$status
    EOF
    chmod +x $out/bin/commit-tf-lint
  '';

  meta = {
    description = "TFLint plus commit-tf-lint, a lint of every OpenTofu configuration";
    homepage = "https://github.com/terraform-linters/tflint";
    license = lib.licenses.mpl20;
    mainProgram = "commit-tf-lint";
  };
}
