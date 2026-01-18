import tomllib
import hashlib
import os
import sys
from pathlib import Path
from mutagen.flac import FLAC

from cli.update.resolver import setup_registry, find_resolver, get_registered_keys
from cli.update.zipper import scan_physical_spine, zip_tracks, parse_int
from cli.update.writer import write_lock
from cli.generate.compressor import get_layout_keys

def validate_layout(config):
    """
    Validates that [lock.layout] only uses keys registered in the system.
    Returns the master lists of keys to calculate.
    """
    # 1. Retrieve Available Keys from Registry
    A_TAGS, A_HELPERS, T_TAGS, T_HELPERS = get_registered_keys()

    # 2. Build Allowed Sets
    allowed_album = set(A_TAGS) | set(A_HELPERS)
    allowed_tracks = set(T_TAGS) | set(T_HELPERS)

    # 3. Retrieve Requested Keys from Layout
    layout_cfg = config.get("lock", {}).get("layout", {})
    layout_album_keys = get_layout_keys(layout_cfg.get("album", []))
    layout_track_keys = get_layout_keys(layout_cfg.get("tracks", []))
    
    # 4. Compare
    unknown_album = layout_album_keys - allowed_album
    unknown_tracks = layout_track_keys - allowed_tracks
    
    if unknown_album:
        print(f"Compiler Error: [lock.layout.album] contains unknown keys: {unknown_album}")
        print("Ensure these keys are defined in Standard Library or registered Extensions.")
        sys.exit(1)
        
    if unknown_tracks:
        print(f"Compiler Error: [lock.layout.tracks] contains unknown keys: {unknown_tracks}")
        print("Ensure these keys are defined in Standard Library or registered Extensions.")
        sys.exit(1)

    return A_TAGS, A_HELPERS, T_TAGS, T_HELPERS

def compile_album(album_root: Path, supported_exts: list, library_root: Path = None):
    # --- CONFIG & REGISTRY SETUP ---
    config_path = Path("config.toml")
    if not config_path.exists():
        return 
        
    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    # Init Registry (only runs once)
    ext_folder = config.get("compiler", {}).get("extensions_folder")
    ext_config = config.get("compiler", {}).get("extensions", {})
    setup_registry(ext_folder, ext_config)

    # Build & Validate Lists
    A_TAGS, A_HELPERS, T_TAGS, T_HELPERS = validate_layout(config)
    
    # Standard Setup
    if not library_root:
        library_root = album_root.parent 

    meta_path = album_root / "metadata.toml"
    
    # --- PREAMBLE: Hashing & Mtime ---
    try:
        meta_mtime = int(os.path.getmtime(meta_path))
    except OSError:
        meta_mtime = 0

    sha256 = hashlib.sha256()
    with open(meta_path, "rb") as f:
        raw_meta = tomllib.load(f)
        f.seek(0)
        while chunk := f.read(8192):
            sha256.update(chunk)
    meta_hash = sha256.hexdigest()

    # --- PHASE 1: THE SPINE ---
    physical_spine = scan_physical_spine(album_root, supported_exts)

    # --- PHASE 2: INFLATION ---
    album_defaults = raw_meta.get("album", {})
    raw_tracks_source = raw_meta.get("tracks", [])
    
    inflated_tracks = []
    
    if not raw_tracks_source and physical_spine:
        for _ in physical_spine:
            inflated_tracks.append(album_defaults.copy())
    else:
        for t in raw_tracks_source:
            inflated_tracks.append({**album_defaults, **t})

    # --- PHASE 3: DISCNUMBER & TRACKNUMBER RESOLUTION ---
    disc_counts = {}
    for track in inflated_tracks:
        if "DISCNUMBER" not in track: track["DISCNUMBER"] = "1"
        dn_str = str(track["DISCNUMBER"]).split('/')[0].strip()
        track["DISCNUMBER"] = dn_str
        
        current_count = disc_counts.get(dn_str, 0) + 1
        disc_counts[dn_str] = current_count
        
        if "TRACKNUMBER" in track:
             track["TRACKNUMBER"] = str(track["TRACKNUMBER"]).split('/')[0].strip()
        else:
             track["TRACKNUMBER"] = str(current_count)

    seen_pairs = set()
    for track in inflated_tracks:
        pair = (parse_int(track.get("DISCNUMBER")), parse_int(track.get("TRACKNUMBER")))
        if pair in seen_pairs:
            raise ValueError(f"Duplicate Disc/Track in {album_root}")
        seen_pairs.add(pair)

    inflated_tracks.sort(key=lambda t: (parse_int(t["DISCNUMBER"]), parse_int(t["TRACKNUMBER"])))

    # --- PHASE 4: TRACK_PATH RESOLUTION (ZIPPER) & NORMALIZATION ---
    zip_tracks(inflated_tracks, physical_spine)
    
    curr_disc = None
    curr_idx = 0
    for track in inflated_tracks:
        d = parse_int(track["DISCNUMBER"])
        if d != curr_disc:
            curr_disc = d
            curr_idx = 0
        curr_idx += 1
        track["TRACKNUMBER"] = str(curr_idx)

    # --- PHASE 5: STANDARD KEY RESOLUTION ---
    
    # A. TRACK RESOLUTION
    final_tracks = []
    
    for track_source in inflated_tracks:
        final_track = {}
        t_path_rel = track_source.get("track_path", "")
        t_path_abs = (album_root / t_path_rel) if t_path_rel else None
        
        audio_obj = None
        if t_path_abs and t_path_abs.exists():
            try: audio_obj = FLAC(t_path_abs)
            except: pass

        track_ctx = {
            "source": track_source,
            "album_root": album_root,
            "library_root": library_root,
            "track_path_resolved": t_path_abs,
            "audio_obj": audio_obj
        }

        # Resolve Tags
        for key in T_TAGS:
            resolver = find_resolver(key, "TRACK_TAGS")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                final_track[key] = str(track_source.get(key, ""))

        # Resolve Helpers
        for key in T_HELPERS:
            resolver = find_resolver(key, "TRACK_HELPERS")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                final_track[key] = ""

        if audio_obj: del audio_obj
        final_tracks.append(final_track)

    # B. ALBUM RESOLUTION
    final_album = {}
    unique_discs = set(t["DISCNUMBER"] for t in final_tracks)
    
    album_ctx = {
        "source": album_defaults,
        "album_root": album_root,
        "library_root": library_root,
        "metadata_toml_hash": meta_hash,
        "metadata_toml_mtime": meta_mtime,
        "total_tracks_count": len(final_tracks),
        "total_discs_count": len(unique_discs),
        "all_tracks_final": final_tracks
    }

    for key in A_TAGS:
        resolver = find_resolver(key, "ALBUM_TAGS")
        if resolver:
            final_album[key] = resolver(album_ctx)
        else:
            final_album[key] = str(album_defaults.get(key, ""))

    for key in A_HELPERS:
        resolver = find_resolver(key, "ALBUM_HELPERS")
        if resolver:
            final_album[key] = resolver(album_ctx)
        else:
            final_album[key] = ""

    # --- PHASE 6: OUTPUT ---
    layout_cfg = config.get("lock", {}).get("layout", {})
    write_lock(album_root, final_album, final_tracks, layout_cfg)
