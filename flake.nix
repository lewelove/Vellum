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

        lyricsgenius = ps: ps.buildPythonPackage rec {
          pname = "lyricsgenius";
          version = "3.7.6";
          pyproject = true;
          
          src = ps.fetchPypi {
            inherit pname version;
            hash = "sha256-zQGrgZEz4o9RSYWmGXH8TcNXUcRSfmF+xJCROQ3cPJ4=";
          };

          nativeBuildInputs =[
            ps.hatchling
          ];

          propagatedBuildInputs =[
            ps.requests
            ps.beautifulsoup4
          ];

          doCheck = false;
        };

        pythonEnv = pkgs.python3.withPackages (ps: with ps;[
          mutagen
          tqdm
          pillow
          numpy
          xxhash
          httpx
          opencv4
          matplotlib
          (lyricsgenius ps)
        ]);

        vellum-cli = pkgs.writeShellApplication {
          name = "vellum";
          runtimeInputs =[ 
            pythonEnv 
            pkgs.bun
            pkgs.cargo 
            pkgs.rustc 
            pkgs.clippy
            pkgs.rustfmt
            pkgs.cargo-deny
            pkgs.pkg-config 
            pkgs.openssl 
            pkgs.nix
            pkgs.git
            pkgs.vulkan-loader
          ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            BIN="$ROOT/rust/target/release/vellum"
            COMMAND=''${1:-"help"}
            if [ "$#" -gt 0 ]; then shift; fi

            case "$COMMAND" in
              build)
                cd "$ROOT/rust" && cargo build --release
                ;;
              ui)
                cd "$ROOT/ui" && bun run dev
                ;;
              server|compile|update|harvest)
                if [ ! -f "$BIN" ]; then
                  echo "Error: vellum binary not found at $BIN. Run 'vellum build' first."
                  exit 1
                fi
                "$BIN" "$COMMAND" "$@"
                ;;
              generate)
                cd "$ROOT" && python -m python.generate "$@"
                ;;
              write)
                cd "$ROOT" && python -m python.write "$@"
                ;;
              report)
                cd "$ROOT" && python -m python.report "$@"
                ;;
              test)
                cd "$ROOT/rust"
                TEST_ARGS=()
                for arg in "$@"; do
                  case "$arg" in
                    --lint) cargo clippy --all-targets --all-features -- -D warnings ;;
                    --fmt)  cargo fmt --all -- --check ;;
                    --deny) cargo deny check ;;
                    *)      TEST_ARGS+=("$arg") ;;
                  esac
                done
                cargo test "''${TEST_ARGS[@]}"
                ;;
              help|--help|-h)
                echo "Vellum CLI Commands:"
                echo "  build           : Build the Rust backend binary"
                echo "  ui              : Start Svelte UI Dev Server"
                echo "  server          : Start Backend Rust Server"
                echo "  compile         : Compile metadata locks"
                echo "  update          : Update library"
                echo "  generate        : Initialize metadata from files"
                echo "  harvest         : Harvest raw metadata to JSON"
                echo "  write           : Sync metadata to audio tags"
                echo "  report          : Generate listening reports"
                echo "  test [flags]    : Run tests"
                echo "    --lint        : Run clippy with -D warnings"
                echo "    --fmt         : Run fmt check"
                echo "    --deny        : Run cargo-deny check"
                ;;
              *)
                echo "Error: Unknown command '$COMMAND'"
                exit 1
                ;;
            esac
          '';
        };

        devPackages = with pkgs;[
          pythonEnv
          bun
          pkg-config
          openssl
          vellum-cli
          cargo
          rustc
          rust-analyzer
          clippy
          rustfmt
          cargo-deny
          libGL
          glib
          gtk3
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          wayland
          libxkbcommon
          fontconfig
          vulkan-loader
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;
          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath[ 
              pkgs.libGL 
              pkgs.glib 
              pkgs.gtk3 
              pkgs.xorg.libX11 
              pkgs.xorg.libXcursor 
              pkgs.xorg.libXrandr 
              pkgs.xorg.libXi 
              pkgs.wayland 
              pkgs.libxkbcommon 
              pkgs.fontconfig 
              pkgs.vulkan-loader
            ]}:$LD_LIBRARY_PATH"

            export PYTHONDONTWRITEBYTECODE=1
            export PATH="$PWD/ui/node_modules/.bin:$PATH"
          '';
        };
      }
    );
}
