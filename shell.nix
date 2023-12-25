{ pkgs ? import <nixpkgs> { } }:
let
  libPath = with pkgs; lib.makeLibraryPath [
    libGL
    libxkbcommon
    wayland
    vulkan-tools
    vulkan-loader
    # alsa-lib
    # xorg.libX11
    # xorg.libXcursor
    # xorg.libXi
    # xorg.libXrandr
  ];
in
with pkgs; mkShell {
  inputsFrom = [ ];
  buildInputs = [ rustup vulkan-tools openssl ];
  LD_LIBRARY_PATH = "${libPath}";
}