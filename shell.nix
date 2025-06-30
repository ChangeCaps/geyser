{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    pkgs.vulkan-loader
    pkgs.vulkan-validation-layers
    pkgs.wayland
    pkgs.libxkbcommon
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
}
