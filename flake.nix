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

          nativeBuildInputs = [
            ps.hatchling
          ];

          propagatedBuildInputs = [
            ps.requests
            ps.beautifulsoup4
          ];

          doCheck = false;
        };

        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          mutagen
          tqdm
          pillow
          numpy
          xxhash
          httpx
          (lyricsgenius ps)
        ]);

        vellum-cli = pkgs.writeShellApplication {
          name = "vellum";
          runtimeInputs = [ 
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
          ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            COMMAND=''${1:-"help"}
            if [ "$#" -gt 0 ]; then shift; fi

            case "$COMMAND" in
              ui)
                cd "$ROOT/ui" && bun run dev
                ;;
              server)
                cd "$ROOT/rust" && cargo run --release -- server "$@"
                ;;
              compile)
                cd "$ROOT/rust" && cargo run --release -- compile "$@"
                ;;
              update)
                cd "$ROOT/rust" && cargo run --release -- update "$@"
                ;;
              generate)
                cd "$ROOT" && python -m python.generate "$@"
                ;;
              harvest)
                cd "$ROOT/rust" && cargo run --release -- harvest "$@"
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
                echo "  ui              : Start Svelte UI Dev Server"
                echo "  server          : Start Backend Rust Server"
                echo "  test [flags]    : Run tests"
                echo "    --lint        : Run clippy with -D warnings"
                echo "    --fmt         : Run fmt check"
                echo "    --deny        : Run cargo-deny check"
                echo "  update          : Update library"
                echo "  generate        : Initialize metadata from files"
                echo "  harvest         : Harvest raw metadata to JSON"
                echo "  write           : Sync metadata to audio tags"
                ;;
              *)
                echo "Error: Unknown command '$COMMAND'"
                exit 1
                ;;
            esac
          '';
        };

        devPackages = with pkgs; [
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
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;
          shellHook = ''
            export PYTHONDONTWRITEBYTECODE=1
            export PATH="$PWD/ui/node_modules/.bin:$PATH"
          '';
        };
      }
    );
}
