# Packages for the development shell, shared by flake.nix and shell.nix.
{ pkgs }:
{
  tools = with pkgs; [
    # Rust (root workspace and pkgs/*)
    cargo
    rustc
    rustfmt
    clippy
    # Go (go/ and pkgs/*)
    go
    # JavaScript (web/, pkgs/*; Yarn for pkgs/portal via corepack)
    nodejs_24
    # Repository tooling
    git
    gh
    jq
    just
    lefthook
    treefmt
    nixfmt
    shellcheck
    pkg-config
  ];

  # System libraries some projects link against on Linux:
  # visualizer (bevy) and desktop (Tauri).
  nativeLibs = pkgs.lib.optionals pkgs.stdenv.isLinux (
    with pkgs;
    [
      alsa-lib
      udev
      wayland
      libxkbcommon
      xorg.libX11
      webkitgtk_4_1
      librsvg
      libayatana-appindicator
      xdotool
    ]
  );
}
