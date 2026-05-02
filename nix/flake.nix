{
  description = "Vellum Core Toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    lib = {
      splitCueImage = { name ? "split", cue, image }: pkgs.stdenv.mkDerivation {
        inherit name cue image;
        buildInputs = [ pkgs.shntool pkgs.cuetools pkgs.flac ];
        unpackPhase = "true";
        buildPhase = ''
          mkdir -p $out
          shnsplit -f "$cue" -o flac -t "%n" -d $out "$image"
        '';
        installPhase = "true";
      };

      mkTrack = { name, src, metadata ? {} }: let
        metaJson = pkgs.writeText "meta.json" (builtins.toJSON metadata);
      in pkgs.stdenv.mkDerivation {
        inherit name src;
        buildInputs = [ pkgs.flac pkgs.jq ];
        unpackPhase = "true";
        buildPhase = ''
          cp "$src" track.flac
          chmod +w track.flac
          metaflac --remove-all-tags track.flac
          jq -r 'to_entries | .[] | if (.value | type) == "array" then .key as $k | .value[] | "\($k)=\(.)" else "\(.key)=\(.value)" end' ${metaJson} > tags.txt
          while IFS= read -r tag; do
            metaflac --set-tag="$tag" track.flac
          done < tags.txt
        '';
        installPhase = ''
          mkdir -p $out
          cp track.flac $out/track.flac
        '';
      };

      mkAlbum = { 
        pname, 
        sourceDisk ? null,
        sha256 ? null,
        sourceTorrent ? null,
        sourceMagnet ? null,
        sourceUrl ? null,
        album ? { metadata = {}; },
        tracks ? [], 
        cover ? null
      }: let
        rawSrc = if (builtins.getEnv "VELLUM_STAGING_SRC") != "" then
                  /. + (builtins.getEnv "VELLUM_STAGING_SRC")
                 else if sourceDisk != null then 
                  (if builtins.isString sourceDisk && builtins.substring 0 1 sourceDisk == "/" then /. + sourceDisk else sourceDisk)
                 else 
                  throw "sourceDisk required or VELLUM_STAGING_SRC must be set";
                  
        realSrc = if sha256 != null 
                  then builtins.path { name = "${pname}-source"; path = rawSrc; inherit sha256; }
                  else builtins.path { name = "${pname}-source"; path = rawSrc; };
        
        builtTracks = pkgs.lib.lists.imap1 (idx: track: let
          mergedMeta = album.metadata // (track.metadata or {});
          trackName = "${pname}-track-${toString idx}";
        in self.lib.mkTrack {
          name = trackName;
          src = realSrc + "/${track.file}";
          metadata = mergedMeta;
        }) tracks;

        tomlFormat = pkgs.formats.toml {};
        metadataToml = tomlFormat.generate "metadata.toml" {
          album = album.metadata;
          tracks = pkgs.lib.lists.map (t: t.metadata or {}) tracks;
        };

        coverExt = if cover != null && cover ? file then pkgs.lib.strings.concatStringsSep "." (pkgs.lib.lists.tail (pkgs.lib.strings.splitString "." (baseNameOf cover.file))) else "jpg";
        coverSrc = if cover != null && cover ? file 
                   then (if cover ? sha256 
                         then builtins.path { name = "${pname}-cover"; path = cover.file; sha256 = cover.sha256; } 
                         else builtins.path { name = "${pname}-cover"; path = cover.file; }) 
                   else null;

      in pkgs.stdenv.mkDerivation {
        name = pname;
        src = realSrc;
        unpackPhase = "true";
        buildPhase = ''
          mkdir -p $out
          ${pkgs.lib.strings.concatImapStringsSep "\n" (idx: trackDrv: ''
            index_padded=$(printf "%02d" ${toString idx})
            ln -s ${trackDrv}/track.flac $out/''${index_padded}.flac
          '') builtTracks}
          cp ${metadataToml} $out/metadata.toml
          ${if coverSrc != null then "cp ${coverSrc} $out/cover.${coverExt}" else ""}
        '';
        installPhase = "true";
      };
    };
  };
}
