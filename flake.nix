{
  description = "Eluxum Development Environment";

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
          xxhash
          orjson
          httpx
          websockets
          uvloop
        ]);

        eluxum-cli = pkgs.writeShellApplication {
          name = "eluxum";
          runtimeInputs = [ pythonEnv pkgs.nodejs_22 pkgs.git pkgs.cargo pkgs.rustc pkgs.pkg-config pkgs.openssl ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            COMMAND=''${1:-"help"}
            if [ "$#" -gt 0 ]; then shift; fi

            case "$COMMAND" in
              ui)
                cd "$ROOT/ui" && npm run dev
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
              generate_rs)
                cd "$ROOT/cli_rs" && cargo run --release -- "$@"
                ;;
              export)
                cd "$ROOT" && python -m cli.export "$@"
                ;;
              help|--help|-h)
                echo "Eluxum CLI Commands:"
                echo "  ui          : Start UI Dev Server"
                echo "  server      : Start Backend (Live State Manager)"
                echo "  update      : Compile metadata locks & Hot Reload Server"
                echo "  generate    : Initialize metadata from files (Python)"
                echo "  generate_rs : Initialize metadata from files (Rust)"
                echo "  export      : Export snapshot"
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
          nodejs_22
          pkg-config
          openssl
          eluxum-cli
          cargo
          rustc
          rust-analyzer
          clippy
          rustfmt
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
