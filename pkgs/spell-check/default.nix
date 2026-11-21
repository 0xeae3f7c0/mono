# Finds and fixes common misspellings in source code, docs and commit
# messages with typos.
# Run `commit-spell-check [path...]` (defaults to the current directory).
{
  lib,
  rustPlatform,
  fetchFromGitHub,
}:

rustPlatform.buildRustPackage rec {
  pname = "typos";
  version = "1.52.2";

  src = fetchFromGitHub {
    owner = "crate-ci";
    repo = "typos";
    rev = "v${version}";
    hash = "sha256-3Esl98ZWK8bppTBpWsJ24DB4C+pjAfSLb1p+f5N9e8k=";
  };

  cargoHash = "sha256-WwKhoGaa/lsJnt48quDB/Jen8EWdFJel0L3anykOVZs=";
  cargoBuildFlags = [
    "-p"
    "typos-cli"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-spell-check <<EOF
    #!/bin/sh
    # Report misspellings; pass --write-changes to fix them in place.
    exec $out/bin/typos "\$@"
    EOF
    chmod +x $out/bin/commit-spell-check
  '';

  meta = {
    description = "typos plus commit-spell-check, a whole-repo spell check";
    homepage = "https://github.com/crate-ci/typos";
    license = with lib.licenses; [
      asl20
      mit
    ];
    mainProgram = "commit-spell-check";
  };
}
