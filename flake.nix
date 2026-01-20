{
  description = "MPF2K Monorepo Development Environment";

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
        ]);

        mpf2k-cli = pkgs.writeShellApplication {
          name = "mpf2k";
          runtimeInputs = [ pythonEnv pkgs.nodejs_22 pkgs.git ];
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
              build)
                cd "$ROOT" && python -m cli.build "$@"
                ;;
              generate)
                cd "$ROOT" && python -m cli.generate "$@"
                ;;
              export)
                cd "$ROOT" && python -m cli.export "$@"
                ;;
              help|--help|-h)
                echo "MPF2K CLI Commands:"
                echo "  ui       : Start UI Dev Server"
                echo "  server   : Start Backend"
                echo "  update   : Compile metadata locks"
                echo "  build    : Aggregate locks into library.json"
                echo "  generate : Initialize metadata from files"
                echo "  export   : Export snapshot"
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
          mpf2k-cli
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
