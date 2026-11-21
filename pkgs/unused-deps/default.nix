# Finds dependencies declared in Cargo.toml that the code never uses, across
# every Rust project in the monorepo, with cargo-machete.
# Run `commit-unused-deps [--fix]` from the repository root.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
}:

rustPlatform.buildRustPackage rec {
  pname = "cargo-machete";
  version = "0.10.1";

  src = fetchFromGitHub {
    owner = "bnjbvr";
    repo = "cargo-machete";
    rev = "v${version}";
    hash = "sha256-EwfyIqwoZgxBPrC7OItztT3HWJyls1N/4r05AcC1334=";
  };

  cargoHash = "sha256-Fnje7FbIbfVFpb+UvyKN2P7aoxUGLrmTPsea6nbM8I8=";

  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-unused-deps <<EOF
    #!/bin/sh
    # Scan the whole repository; --fix removes the unused entries.
    exec $out/bin/cargo-machete "\$@" .
    EOF
    chmod +x $out/bin/commit-unused-deps
  '';

  meta = {
    description = "cargo-machete plus commit-unused-deps, an unused Rust dependency check";
    homepage = "https://github.com/bnjbvr/cargo-machete";
    license = lib.licenses.mit;
    mainProgram = "commit-unused-deps";
  };
}
