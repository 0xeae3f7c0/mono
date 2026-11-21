# The repository's own scripts as commands with pinned dependencies, so they
# behave the same locally and in CI: commit-gen, commit-deps, commit-labels and
# commit-update-deps. Run them from the repository root.
{
  lib,
  symlinkJoin,
  writeShellApplication,
  nodejs_24,
  jq,
  gh,
  git,
  coreutils,
  gnused,
  gnugrep,
  findutils,
  cargo,
  go,
  curl,
}:

let
  tool =
    name: description: runtimeInputs: text:
    writeShellApplication {
      inherit name runtimeInputs text;
      meta.description = description;
    };
in
symlinkJoin {
  name = "repo-tools";
  paths = [
    (tool "commit-gen" "Regenerate (or --check) workflows, labeler config and the portal catalog"
      [ nodejs_24 ]
      ''
        node scripts/gen-pkg-workflows.mjs "$@"
        node scripts/gen-catalog.mjs "$@"
      ''
    )
    (tool "commit-deps" "Print unique locked dependencies per ecosystem" [ nodejs_24 ] ''
      node scripts/count-deps.mjs . "$@"
    '')
    (tool "commit-labels" "Print (or --sync) the repository labels" [ nodejs_24 ] ''
      node scripts/labels.mjs "$@"
    '')
    (tool "commit-update-deps" "Open one pull request per available dependency update"
      [
        nodejs_24
        jq
        gh
        git
        coreutils
        gnused
        gnugrep
        findutils
        cargo
        go
        curl
      ]
      ''
        bash scripts/update-deps.sh "$@"
      ''
    )
  ];

  meta = {
    description = "Repository scripts with pinned dependencies";
    mainProgram = "commit-gen";
    platforms = lib.platforms.unix;
  };
}
