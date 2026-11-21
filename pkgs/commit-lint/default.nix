# Checks that commit messages follow Conventional Commits (the format
# release-please and the PR title check rely on) with committed.
# Run `commit-commit-lint [revision-range]` (defaults to origin/main..HEAD).
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  libgit2,
  zlib,
}:

rustPlatform.buildRustPackage rec {
  pname = "committed";
  version = "1.1.12";

  src = fetchFromGitHub {
    owner = "crate-ci";
    repo = "committed";
    rev = "v${version}";
    hash = "sha256-giIBcqS5n2SKs8Pc9GInzZakq4uMTrR9nNussDLuPqE=";
  };

  cargoHash = "sha256-YQh8MacpfMCNV4ipwTV/eJ4pn9+TMvc3eZrPFoD+66s=";

  # Link the system libgit2 instead of building the bundled copy.
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [
    libgit2
    zlib
  ];
  env.LIBGIT2_NO_VENDOR = 1;
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-commit-lint <<EOF
    #!/bin/sh
    # Check every commit in the range against Conventional Commits.
    exec $out/bin/committed "\''${1:-origin/main..HEAD}"
    EOF
    chmod +x $out/bin/commit-commit-lint
  '';

  meta = {
    description = "committed plus commit-commit-lint, a Conventional Commits check";
    homepage = "https://github.com/crate-ci/committed";
    license = with lib.licenses; [
      asl20
      mit
    ];
    mainProgram = "commit-commit-lint";
  };
}
