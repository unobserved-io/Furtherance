{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    cargo
    cargo-bundle
    rustc
    fontconfig
    pkg-config
    cmake
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libXScrnSaver
    wayland
    wayland-protocols
    libxkbcommon
    vulkan-loader
  ];

  LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;

}
