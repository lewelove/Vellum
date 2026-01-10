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

        # --- Python Stack (Logic Layer) ---
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

        # --- System Dependencies ---
        devPackages = with pkgs; [
          pythonEnv
          nodejs_22
          pkg-config
          openssl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;

          # --- Shell Orchestration ---
          shellHook = ''
            export PYTHONDONTWRITEBYTECODE=1
            
            export PATH="$PWD/ui/node_modules/.bin:$PATH"

            echo " Python:  $(python --version)"
            echo " Node:    $(node --version)"
          '';
        };
      }
    );
}
