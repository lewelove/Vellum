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
          # python-multipart is often needed for FastAPI form handling
          python-multipart 
        ]);

        # --- System Dependencies ---
        # Minimalist set for a high-performance web-first workflow
        devPackages = with pkgs; [
          pythonEnv
          nodejs_22
          # Common build tools in case of native node-gyp or python extensions
          pkg-config
          openssl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;

          # --- Shell Orchestration ---
          shellHook = ''
            # Ensure Python doesn't write __pycache__ everywhere in the project
            export PYTHONDONTWRITEBYTECODE=1
            
            # Setup local node_modules bin in PATH for ease of use
            export PATH="$PWD/apps/ui/node_modules/.bin:$PATH"

            echo ""
            echo " 🎵 MPF2K ARCHITECTURAL ENVIRONMENT"
            echo " ----------------------------------"
            echo " Python:  $(python --version)"
            echo " Node:    $(node --version)"
            echo " FastAPI: Ready"
            echo " Svelte:  Ready (Vite)"
            echo ""
            echo " Tip: Run 'npm run dev' from the root to start the engine."
          '';
        };
      }
    );
}
