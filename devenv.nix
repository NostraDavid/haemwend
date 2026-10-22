{
  pkgs,
  lib,
  ...
}: let
  libraries = with pkgs; [
    alsa-lib
    libxkbcommon
    udev
    vulkan-loader
    wayland
    libx11
    libxcursor
    libxi
    libxrandr
  ];
in {
  cachix.enable = false;
  packages = with pkgs;
    [
      rustc
      cargo
      clippy
      rustfmt
      rust-analyzer
      rustPlatform.rustLibSrc
      cargo-watch
      just
      uv
      python314
      blender
      git-lfs
      oxipng
      pngquant
      pkg-config
      openssl
    ]
    ++ libraries;
  env = {
    UV_PYTHON = "3.14";
    UV_PYTHON_PREFERENCE = "only-system";
    UV_PYTHON_DOWNLOADS = "never";
    LD_LIBRARY_PATH = lib.makeLibraryPath libraries;
    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}/lib/rustlib/src/rust/library";
  };
}
