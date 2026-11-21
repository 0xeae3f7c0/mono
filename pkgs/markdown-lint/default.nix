# Lints the monorepo's Markdown (READMEs, docs, changelog) for broken
# structure and style with rumdl.
# Run `commit-markdown-lint [--fix]` from the repository root.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  rust-jemalloc-sys,
}:

rustPlatform.buildRustPackage rec {
  pname = "rumdl";
  version = "0.3.5";

  src = fetchFromGitHub {
    owner = "rvben";
    repo = "rumdl";
    rev = "v${version}";
    hash = "sha256-9WBThxAZi6wgiaFn1DLqyEsm+majY94vgepObrzgApg=";
  };

  cargoHash = "sha256-PFNjs6jAnVjbJo97CA42OKBVxVAL/r1tK+jF+pcOtcw=";

  # Use the nixpkgs jemalloc build, as zizmor in pkgs/actions-lint does.
  buildInputs = [ rust-jemalloc-sys ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-markdown-lint <<EOF
    #!/bin/sh
    # Check every Markdown file; --fix applies the safe fixes.
    exec $out/bin/rumdl check "\$@" .
    EOF
    chmod +x $out/bin/commit-markdown-lint
  '';

  meta = {
    description = "rumdl plus commit-markdown-lint, a whole-repo Markdown lint";
    homepage = "https://github.com/rvben/rumdl";
    license = lib.licenses.mit;
    mainProgram = "commit-markdown-lint";
  };
}
