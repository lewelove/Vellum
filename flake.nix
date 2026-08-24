{
  description = "Dale Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        runtimeLibs = with pkgs; [
          libGL
          libxkbcommon
          wayland
          libX11
          libXcursor
          libXi
          libXrandr
        ];

        lyricsgenius = ps: ps.buildPythonPackage rec {
          pname = "lyricsgenius";
          version = "3.7.6";
          pyproject = true;
          
          src = ps.fetchPypi {
            inherit pname version;
            hash = "sha256-zQGrgZEz4o9RSYWmGXH8TcNXUcRSfmF+xJCROQ3cPJ4=";
          };

          nativeBuildInputs = [
            ps.hatchling
          ];

          propagatedBuildInputs = [
            ps.requests
            ps.beautifulsoup4
          ];

          doCheck = false;
        };

        get_lyrics = pkgs.writers.writePython3Bin "get_lyrics" {
          libraries = [
            (lyricsgenius pkgs.python3Packages)
            pkgs.python3Packages.requests
            pkgs.python3Packages.beautifulsoup4
          ];
          doCheck = false;
        } (builtins.readFile ./actions/python/get_lyrics/main.py);

        search_cover = pkgs.writers.writePython3Bin "search_cover" {
          libraries = [];
          doCheck = false;
        } (builtins.readFile ./actions/python/search_cover/main.py);

        embed = pkgs.writers.writePython3Bin "embed" {
          libraries = [
            pkgs.python3Packages.mutagen
            pkgs.python3Packages.xxhash
          ];
          doCheck = false;
        } (builtins.readFile ./actions/python/embed/main.py);

        rename = pkgs.writers.writePython3Bin "rename" {
          libraries = [];
          doCheck = false;
        } (builtins.readFile ./actions/python/rename/main.py);

        build-cli = pkgs.writeShellApplication {
          name = "build";
          runtimeInputs = [ pkgs.cargo pkgs.rustc pkgs.git pkgs.clippy pkgs.nix pkgs.bun ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            TARGET=""
            ARGS=()

            for arg in "$@"; do
              case "$arg" in
                libdale)    TARGET="libdale" ;;
                dale)       TARGET="dale" ;;
                actions)    TARGET="actions" ;;
                interface)  TARGET="interface" ;;
                web-app)    TARGET="interface" ;;
                *)          ARGS+=("$arg") ;;
              esac
            done

            build_dale() {
              echo ""
              echo "Checking and linting code..."
              echo ""
              cd "$ROOT"
              cargo clippy -p dale -p libdale -- -D warnings
              echo ""
              echo "Running tests..."
              echo ""
              cargo test -p libdale
              echo ""
              echo "Building \`dale\` Binary..."
              echo ""
              cargo build -p dale --release "''${ARGS[@]}"
            }

            build_libdale() {
              cd "$ROOT"
              cargo clippy -p libdale -- -D warnings
              cargo test -p libdale
              cargo build -p libdale --release "''${ARGS[@]}"
            }

            build_actions() {
              echo ""
              echo "Building Actions..."
              echo ""

              cd "$ROOT"
              cargo clippy -p libactions -p get_theme -p collect -p discogs_fetch_master -p musicbrainz_search -p calculate_cover_metrics -- -D warnings
              cargo build -p get_theme -p collect -p discogs_fetch_master -p musicbrainz_search -p calculate_cover_metrics --release "''${ARGS[@]}"
              
              nix build .#get_lyrics --out-link actions/python/get_lyrics/result
              nix build .#search_cover --out-link actions/python/search_cover/result
              nix build .#embed --out-link actions/python/embed/result
              nix build .#rename --out-link actions/python/rename/result
              
              ln -sf "../target/release/get_theme" "$ROOT/actions/get_theme"
              ln -sf "../target/release/collect" "$ROOT/actions/collect"
              ln -sf "../target/release/discogs_fetch_master" "$ROOT/actions/discogs_fetch_master"
              ln -sf "../target/release/musicbrainz_search" "$ROOT/actions/musicbrainz_search"
              ln -sf "../target/release/calculate_cover_metrics" "$ROOT/actions/calculate_cover_metrics"
              ln -sf "python/get_lyrics/result/bin/get_lyrics" "$ROOT/actions/get_lyrics"
              ln -sf "python/search_cover/result/bin/search_cover" "$ROOT/actions/search_cover"
              ln -sf "python/embed/result/bin/embed" "$ROOT/actions/embed"
              ln -sf "python/rename/result/bin/rename" "$ROOT/actions/rename"
            }

            build_interface() {
              echo ""
              echo "Building Web App Interface..."
              echo ""

              cd "$ROOT/interfaces/web-app"
              bun run build
            }

            if [ "$TARGET" = "dale" ]; then
              build_dale
            elif [ "$TARGET" = "libdale" ]; then
              build_libdale
            elif [ "$TARGET" = "actions" ]; then
              build_actions
            elif [ "$TARGET" = "interface" ]; then
              build_interface
            else
              build_dale
              build_actions
              build_interface
            fi
          '';
        };

        check-cli = pkgs.writeShellApplication {
          name = "check";
          runtimeInputs = [ pkgs.cargo pkgs.rustc pkgs.git pkgs.clippy ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            ARGS=()
            LINT=false

            for arg in "$@"; do
              case "$arg" in
                --lint) LINT=true ;;
                *)      ARGS+=("$arg") ;;
              esac
            done

            cd "$ROOT"
            if [ "$LINT" = true ]; then
              cargo clippy --workspace -- "''${ARGS[@]}" -D warnings
            else
              cargo check --workspace "''${ARGS[@]}"
            fi
          '';
        };

        dale-bin = pkgs.writeShellApplication {
          name = "dale";
          runtimeInputs = [ 
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
          ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            BIN="$ROOT/target/release/dale"
            COMMAND=''${1:-"help"}
            if [ "$#" -gt 0 ]; then shift; fi

            case "$COMMAND" in
              interface|server|manifest|compile|update|harvest|x|query)
                if [ ! -f "$BIN" ]; then
                  echo "Error: dale binary not found at $BIN. Run 'build dale --release' first."
                  exit 1
                fi
                cd "$ROOT" && "$BIN" "$COMMAND" "$@"
                ;;
              test)
                TEST_ARGS=()
                for arg in "$@"; do
                  case "$arg" in
                    --lint) cargo clippy --workspace --all-targets --all-features -- -D warnings ;;
                    --fmt)  cargo fmt --all -- --check ;;
                    --deny) cargo deny check ;;
                    *)      TEST_ARGS+=("$arg") ;;
                  esac
                done

                cd "$ROOT"
                cargo test --workspace "''${TEST_ARGS[@]}"
                ;;
              help|--help|-h)
                echo "Dale CLI Commands:"
                echo "  interface       : Run system installed interface"
                echo "  server          : Start Backend Rust Server"
                echo "  compile         : Compile metadata locks"
                echo "  update          : Update library"
                echo "  query           : Run SQL queries against the library"
                echo "  harvest         : Harvest raw metadata to JSON"
                echo "  x               : Run defined actions via runtime router"
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

        devPackages = with pkgs; [
          bun
          pkg-config
          openssl
          build-cli
          check-cli
          dale-bin
          cargo
          rustc
          rust-analyzer
          clippy
          rustfmt
          cargo-deny
          glib
          gtk3
        ] ++ runtimeLibs;
      in
      {
        packages.get_lyrics = get_lyrics;
        packages.search_cover = search_cover;
        packages.embed = embed;
        packages.rename = rename;

        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;
          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH"
            export PATH="$PWD/interfaces/web-app/node_modules/.bin:$PATH"
          '';
        };
      }
    );
}
