{
  description = "Vellum Extensions Runtime Overlay";

  inputs = {
    # Inherit from the root vellum flake to keep environment consistent
    vellum-root.url = "path:../";
    
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, vellum-root }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rootShell = vellum-root.devShells.${system}.default;
      in
      {
        devShells.default = pkgs.mkShell {
          # Inherit inputs from the root shell (includes bun, cargo, etc.)
          inputsFrom = [ rootShell ];

          # Add runtime-specific tools that extensions might need
          buildInputs = with pkgs; [
            ffmpeg          # Audio analysis
            imagemagick     # Image processing
            fpcalc          # AcoustID fingerprinting
            yt-dlp          # Metadata fetching
          ];
        };
      }
    );
}
