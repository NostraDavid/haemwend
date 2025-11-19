# This flake provides a development shell
# I use it to setup a .venv - once that's up, you can do whatever you want.
# I auto-activate the .venv using some private bash scripts
# run commands for this file:
# file inspiration: https://pyproject-nix.github.io/uv2nix/usage/hello-world.html
{
  description = "Development shell (impure venv + uv)";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-unstable,
    ...
  }: let
    system = "x86_64-linux";

    pkgs = import nixpkgs {
      inherit system;
      config.allowUnfree = true;
    };

    pkgs-unstable = import nixpkgs-unstable {
      inherit system;
      config.allowUnfree = true;
    };

    pyPkgs = pkgs-unstable;
    python = pyPkgs.python313;
    uv = pyPkgs.uv;
  in {
    devShells.${system}.default = pkgs.mkShell {
      # Provide locales without invoking nix-env in shell
      LOCALE_ARCHIVE = "${pkgs.glibcLocales}/lib/locale/locale-archive";

      packages = [
        python
        uv
        pkgs.xorg.libX11
        pkgs.libGL
        pkgs.xorg.libXrender
        pkgs.xorg.libXext
        pkgs.xorg.libXcursor
        pkgs.xorg.libXrandr
        pkgs.xorg.libXi
        pkgs.fontconfig
        pkgs.freetype
      ];

      env = {
        UV_PYTHON_DOWNLOADS = "never";
        PYTHONNOUSERSITE = "1";
        XDG_UTILS_INSTALL_MODE = "manual";
      };

      shellHook = ''
        export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
          pkgs.xorg.libX11
          pkgs.libGL
          pkgs.xorg.libXrender
          pkgs.xorg.libXext
          pkgs.xorg.libXcursor
          pkgs.xorg.libXrandr
          pkgs.xorg.libXi
          pkgs.fontconfig
          pkgs.freetype
        ]}:$LD_LIBRARY_PATH"
        unset PYTHONPATH

        # Prefer active venv if present
        if [ -n "$VIRTUAL_ENV" ] && [ -x "$VIRTUAL_ENV/bin/python" ]; then
          export UV_PYTHON="$VIRTUAL_ENV/bin/python"
        else
          export UV_PYTHON="${python}/bin/python"
        fi

        # Quick hints for first-time setup
        if [ ! -d ".venv" ]; then
          echo "[hint] No .venv detected. Create one with: uv venv && uv sync"
        fi

        # Print versions defensively (avoid hard failures on one command)
        set +e
        echo -n "Python: "; "${python}/bin/python" -V 2>/dev/null || echo "unavailable"
        echo -n "uv: "; "${uv}/bin/uv" --version 2>/dev/null | head -n1 || echo "unavailable"
        set -e
      '';
    };
  };
}
