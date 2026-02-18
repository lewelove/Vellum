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
          mutagen
          tqdm
          pillow
          numpy
          xxhash
          httpx
        ]);

        vellum-cli = pkgs.writeShellApplication {
          name = "vellum";
          runtimeInputs = [ 
            pythonEnv 
            pkgs.bun
            pkgs.cargo 
            pkgs.rustc 
            pkgs.pkg-config 
            pkgs.openssl 
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
              update)
                cd "$ROOT" && python -m python.update "$@"
                ;;
              generate)
                cd "$ROOT" && python -m python.generate "$@"
                ;;
              harvest)
                cd "$ROOT/rust" && cargo run --release -- harvest "$@"
                ;;
              export)
                cd "$ROOT" && python -m python.export "$@"
                ;;
              write)
                cd "$ROOT" && python -m python.write "$@"
                ;;
              report)
                cd "$ROOT" && python -m python.report "$@"
                ;;
              help|--help|-h)
                echo "Vellum CLI Commands:"
                echo "  ui          : Start Svelte UI Dev Server (Bun)"
                echo "  server      : Start Backend Rust Server"
                echo "  update      : Compile metadata locks"
                echo "  generate    : Initialize metadata from files"
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

        devPackages = with pkgs; [
          pythonEnv
          bun
          pkg-config
          openssl
          vellum-cli
          cargo
          rustc
          rust-analyzer
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
