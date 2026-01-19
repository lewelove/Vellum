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

        # --- Python Stack ---
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

        # --- Unified CLI Wrapper ---
        mpf2k-cli = pkgs.writeShellApplication {
          name = "mpf2k";
          runtimeInputs = [ pythonEnv pkgs.nodejs_22 pkgs.git ];
          text = ''
            ROOT=$(git rev-parse --show-toplevel)
            
            COMMAND=''${1:-"help"}
            
            # Shift the first argument (command) so "$@" contains flags like --force
            if [ "$#" -gt 0 ]; then
              shift
            fi

            case "$COMMAND" in
              ui)
                cd "$ROOT/ui" && npm run dev
                ;;
              server)
                # CHANGED: Run as module (-m) to ensure sys.path includes ROOT
                cd "$ROOT" && python -m server.main "$@"
                ;;
              update)
                cd "$ROOT" && python -m cli.update "$@"
                ;;
              generate)
                cd "$ROOT" && python -m cli.generate "$@"
                ;;
              export)
                cd "$ROOT" && python -m cli.export "$@"
                ;;
              help|--help|-h)
                echo "MPF2K CLI - Available Commands:"
                echo "  ui       : Start Svelte/Vite development server"
                echo "  server   : Start Python FastAPI backend"
                echo "  generate : Run the metadata compiler"
                echo "  update   : Update metadata.lock files"
                echo "  export   : Export merged metadata and assets"
                echo "  help     : Show this help message"
                ;;
              *)
                echo "Error: Unknown command '$COMMAND'"
                echo "Run 'mpf2k help' for usage."
                exit 1
                ;;
            esac
          '';
        };

        # --- System Dependencies ---
        devPackages = with pkgs; [
          pythonEnv
          nodejs_22
          pkg-config
          openssl
          mpf2k-cli
          sqlite
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
