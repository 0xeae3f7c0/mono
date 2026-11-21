# Writes a CycloneDX software bill of materials for the root workspace and
# every project in pkgs/ with Syft: one sbom/<name>.cdx.json per project.
# Run `commit-sbom [output-dir]` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "syft";
  version = "1.57.2";

  src = fetchFromGitHub {
    owner = "anchore";
    repo = "syft";
    rev = "v${version}";
    hash = "sha256-hQOM7sw9f2FaocpVmQL9cewdqQYeoj5SfAplvFg5o3g=";
  };

  vendorHash = "sha256-oG0xEAaEWZ5QYWeD1fwa3dscHq6V3EkpuS97KExfCsA=";

  subPackages = [ "cmd/syft" ];
  ldflags = [
    "-s"
    "-w"
    "-X=main.version=${version}"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-sbom <<EOF
    #!/bin/sh
    # One CycloneDX SBOM per project: the root workspace plus pkgs/*.
    set -eu
    out="\''${1:-sbom}"
    mkdir -p "\$out"
    $out/bin/syft scan dir:. --exclude './pkgs/**' -o "cyclonedx-json=\$out/root.cdx.json" -q
    for dir in pkgs/*/; do
      name=\$(basename "\$dir")
      $out/bin/syft scan "dir:\$dir" -o "cyclonedx-json=\$out/\$name.cdx.json" -q
      echo "\$out/\$name.cdx.json"
    done
    EOF
    chmod +x $out/bin/commit-sbom
  '';

  meta = {
    description = "Syft plus commit-sbom, one CycloneDX SBOM per project";
    homepage = "https://github.com/anchore/syft";
    license = lib.licenses.asl20;
    mainProgram = "commit-sbom";
  };
}
