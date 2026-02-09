{
  description = "Vellum Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          fastapi
          uvicorn
          mpd2
          pyyaml
          watchdog
          python-multipart 
          mutagen
          tqdm
          pillow
          numpy
          xxhash
          orjson
          httpx
          websockets
          uvloop
          pyside6
          websocket-client
        ]);

        vellum-cli = pkgs.writeShellApplication {
          name = "vellum";
          runtimeInputs = [ pythonEnv pkgs.nodejs_22 pkgs.git pkgs.cargo pkgs.rustc pkgs.pkg-config pkgs.openssl ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            COMMAND=''${1:-"help"}
            if [ "$#" -gt 0 ]; then shift; fi

            case "$COMMAND" in
              ui)
                cd "$ROOT/ui" && npm run dev
                ;;
              ui_qml)
                cd "$ROOT" && python ui_qml/main.py "$@"
                ;;
              server)
                cd "$ROOT" && python -m server.main "$@"
                ;;
              update)
                cd "$ROOT" && python -m cli.update "$@"
                ;;
              generate)
                cd "$ROOT" && python -m cli.generate "$@"
                ;;
              harvest)
                cd "$ROOT/cli_rs" && cargo run --release -- harvest "$@"
                ;;
              export)
                cd "$ROOT" && python -m cli.export "$@"
                ;;
              report)
                cd "$ROOT" && python -m cli.report "$@"
                ;;
              help|--help|-h)
                echo "Vellum CLI Commands:"
                echo "  ui          : Start Svelte UI Dev Server"
                echo "  server      : Start Backend (Live State Manager)"
                echo "  update      : Compile metadata locks & Hot Reload Server"
                echo "  generate    : Initialize metadata from files (Python)"
                echo "  harvest     : Harvest raw metadata to JSON (Rust)"
                echo "  export      : Export snapshot"
                echo "  report      : Generate listening report"
                ;;
              *)
                echo "Error: Unknown command '$COMMAND'"
                exit 1
                ;;
            esac
          '';
        };

        qt6Deps = with pkgs.qt6; [
          qtbase
          qtdeclarative
          qtsvg
          qtwayland
        ];

        devPackages = with pkgs; [
          pythonEnv
          nodejs_22
          pkg-config
          openssl
          vellum-cli
          cargo
          rustc
          rust-analyzer
          fontconfig
          libglvnd
          libxkbcommon
          wayland
          libGL
          vulkan-loader
        ] ++ qt6Deps;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;
          shellHook = ''
            export PYTHONDONTWRITEBYTECODE=1
            export PATH="$PWD/ui/node_modules/.bin:$PATH"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath devPackages}:$LD_LIBRARY_PATH"
            export QML2_IMPORT_PATH="${pkgs.qt6.qtdeclarative}/lib/qt-6/qml"
            export QT_PLUGIN_PATH="${pkgs.qt6.qtbase}/lib/qt-6/plugins"
            export QT_QPA_PLATFORM="wayland;xcb"
          '';
        };
      }
    );
}
