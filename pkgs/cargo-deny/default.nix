# Checks every Cargo workspace in the monorepo for security advisories, banned
# or duplicate crates and untrusted sources with cargo-deny.
# Run `commit-cargo-deny` from the repository root.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  cargo,
  pkg-config,
  zstd,
}:

rustPlatform.buildRustPackage rec {
  pname = "cargo-deny";
  version = "0.22.0";

  src = fetchFromGitHub {
    owner = "EmbarkStudios";
    repo = "cargo-deny";
    rev = version;
    hash = "sha256-Yjt2tjGtnmh3Ljf+Ikf27WzTEZ7ge0vL2i0n9oqP+Zo=";
  };

  cargoHash = "sha256-cm5TRk2bqI2CmAbLhtPI/SQKhfmvSMC9S52EXpu+43k=";

  # Link the system zstd instead of building the bundled copy.
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ zstd ];
  env.ZSTD_SYS_USE_PKG_CONFIG = true;
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-cargo-deny <<EOF
    #!/bin/sh
    # cargo-deny reads the dependency graph with cargo metadata.
    export PATH="${lib.makeBinPath [ cargo ]}:\$PATH"
    # Run cargo-deny in each directory that has a Cargo.lock.
    set -u
    status=0
    for lock in \$(git ls-files '*Cargo.lock'); do
      dir=\$(dirname "\$lock")
      echo "== \$dir"
      (cd "\$dir" && $out/bin/cargo-deny check advisories bans sources) || status=1
    done
    exit \$status
    EOF
    chmod +x $out/bin/commit-cargo-deny
  '';

  meta = {
    description = "cargo-deny plus commit-cargo-deny, a check of every Cargo workspace";
    homepage = "https://github.com/EmbarkStudios/cargo-deny";
    license = with lib.licenses; [
      asl20
      mit
    ];
    mainProgram = "commit-cargo-deny";
  };
}
