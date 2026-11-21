# Checks and formats the monorepo's shell scripts (scripts/*.sh and the root
# shortcuts) with shfmt.
# Run `commit-shell-format [--fix]` from the repository root.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:

buildGoModule rec {
  pname = "shfmt";
  version = "3.14.1";

  src = fetchFromGitHub {
    owner = "mvdan";
    repo = "sh";
    rev = "v${version}";
    hash = "sha256-taZfXF8i8Kb6RjW7tWMFfaHH8K4OE4FT+U+X0IBaj08=";
  };

  vendorHash = "sha256-OfdXhgPyRRifRK4CylmOqWh0qmZ80mNAhBZtqJlwPPY=";

  subPackages = [ "cmd/shfmt" ];
  # shfmt --version reads the module version from Go build info, which Nix
  # builds leave as (devel); there is no linker flag for it.
  ldflags = [
    "-s"
    "-w"
  ];
  doCheck = false;

  postInstall = ''
    cat > $out/bin/commit-shell-format <<EOF
    #!/bin/sh
    # Show a diff for every tracked shell script that is not formatted;
    # with --fix, rewrite them in place instead.
    set -eu
    files=\$(git ls-files '*.sh')
    if [ "\''${1:-}" = --fix ]; then
      exec $out/bin/shfmt -i 2 -ci -w \$files
    fi
    exec $out/bin/shfmt -i 2 -ci -d \$files
    EOF
    chmod +x $out/bin/commit-shell-format
  '';

  meta = {
    description = "shfmt plus commit-shell-format, a whole-repo shell format check";
    homepage = "https://github.com/mvdan/sh";
    license = lib.licenses.bsd3;
    mainProgram = "commit-shell-format";
  };
}
