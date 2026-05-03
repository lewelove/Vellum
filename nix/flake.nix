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

      mkTrack = { name, src, metadata ? {}, writeTags ? null }: let
        filteredMeta = if writeTags == null 
                       then metadata 
                       else pkgs.lib.filterAttrs (k: v: builtins.elem k writeTags) metadata;
        metaJson = pkgs.writeText "meta.json" (builtins.toJSON filteredMeta);
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
        tracks ?[], 
        cover ? null,
        writeTags ? null
      }: let
        trackIds = builtins.map (t: "${toString (t.metadata.discnumber or 1)}-${toString (t.metadata.tracknumber or 0)}") tracks;
        uniqueTrackIds = pkgs.lib.unique trackIds;
        hasDuplicates = builtins.length trackIds != builtins.length uniqueTrackIds;

        maxDisc = builtins.foldl' (acc: t: pkgs.lib.max acc (t.metadata.discnumber or 1)) 1 tracks;
        maxTrack = builtins.foldl' (acc: t: pkgs.lib.max acc (t.metadata.tracknumber or 0)) 1 tracks;
        
        discPadLen = pkgs.lib.max 1 (builtins.stringLength (toString maxDisc));
        trackPadLen = pkgs.lib.max 2 (builtins.stringLength (toString maxTrack));

        toTomlVal = v:
          if builtins.isString v then "\"${pkgs.lib.escape ["\"" "\\"] v}\""
          else if builtins.isInt v then toString v
          else if builtins.isBool v then (if v then "true" else "false")
          else if builtins.isList v then "[ " + pkgs.lib.concatMapStringsSep ", " toTomlVal v + " ]"
          else "\"\"";
        
        toTomlTable = attrs: pkgs.lib.concatStringsSep "\n" (pkgs.lib.mapAttrsToList (k: v: "${k} = ${toTomlVal v}") attrs);

        metadataTomlContent = ''
          [album]
          ${toTomlTable album.metadata}

          ${pkgs.lib.concatMapStringsSep "\n\n" (t: ''
            [[tracks]]
            ${toTomlTable (t.metadata or {})}
          '') tracks}
        '';
        metadataToml = pkgs.writeText "metadata.toml" metadataTomlContent;

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
          disc = track.metadata.discnumber or 1;
          trk = track.metadata.tracknumber or 0;
          title = track.metadata.title or "Untitled";

          discStr = pkgs.lib.fixedWidthString discPadLen "0" (toString disc);
          trkStr = pkgs.lib.fixedWidthString trackPadLen "0" (toString trk);
          
          fileName = if maxDisc == 1 then "${trkStr} - ${title}.flac" else "${discStr}.${trkStr} - ${title}.flac";

          mergedMeta = album.metadata // (track.metadata or {});
          trackName = "${pname}-disc${toString disc}-track${toString trk}";
        in {
          inherit fileName;
          drv = self.lib.mkTrack {
            name = trackName;
            src = realSrc + "/${track.file}";
            metadata = mergedMeta;
            inherit writeTags;
          };
        }) tracks;

        coverExt = if cover != null && cover ? file then pkgs.lib.strings.concatStringsSep "." (pkgs.lib.lists.tail (pkgs.lib.strings.splitString "." (baseNameOf cover.file))) else "jpg";
        coverSrc = if cover != null && cover ? file 
                   then (if cover ? sha256 
                         then builtins.path { name = "${pname}-cover"; path = cover.file; sha256 = cover.sha256; } 
                         else builtins.path { name = "${pname}-cover"; path = cover.file; }) 
                   else null;

      in if hasDuplicates then throw "Duplicate discnumber and tracknumber combinations found in tracks." else pkgs.stdenv.mkDerivation {
        name = pname;
        src = realSrc;
        unpackPhase = "true";
        buildPhase = ''
          mkdir -p $out
          ${pkgs.lib.strings.concatMapStringsSep "\n" (t: ''
            ln -s "${t.drv}/track.flac" "$out/${t.fileName}"
          '') builtTracks}
          cp ${metadataToml} $out/metadata.toml
          ${if coverSrc != null then "cp ${coverSrc} $out/cover.${coverExt}" else ""}
        '';
        installPhase = "true";
      };
    };
  };
}
