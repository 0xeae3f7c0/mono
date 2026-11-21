# Python projects in pkgs/ built with uv2nix from their uv.lock: each becomes a
# virtual environment holding the project and its locked dependencies, wrapped
# so its command finds git. Add a project by listing its directory below.
{
  pkgs,
  pyproject-nix,
  uv2nix,
  pyproject-build-systems,
}:
let
  inherit (pkgs) lib;
  python = pkgs.python313;

  build =
    {
      name,
      command,
    }:
    let
      workspace = uv2nix.lib.workspace.loadWorkspace { workspaceRoot = ../pkgs + "/${name}"; };
      pythonSet = (pkgs.callPackage pyproject-nix.build.packages { inherit python; }).overrideScope (
        lib.composeManyExtensions [
          pyproject-build-systems.overlays.wheel
          (workspace.mkPyprojectOverlay { sourcePreference = "wheel"; })
        ]
      );
      venv = pythonSet.mkVirtualEnv "${name}-env" workspace.deps.default;
    in
    pkgs.runCommand name
      {
        nativeBuildInputs = [ pkgs.makeWrapper ];
        meta.mainProgram = command;
      }
      ''
        mkdir -p $out/bin
        makeWrapper ${venv}/bin/${command} $out/bin/${command} \
          --prefix PATH : ${lib.makeBinPath [ pkgs.git ]}
      '';
in
{
  heatmap = build {
    name = "heatmap";
    command = "commit-heatmap";
  };
}
