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
from cli.update.image_processor import generate_thumbnail

def validate_layout(config):
    """
    Validates that [lock.layout] only uses keys registered in the system.
    Returns the master lists of keys to calculate.
    """
    A_TAGS, A_HELPERS, T_TAGS, T_HELPERS = get_registered_keys()

    allowed_album = set(A_TAGS) | set(A_HELPERS)
    allowed_tracks = set(T_TAGS) | set(T_HELPERS)

    layout_cfg = config.get("lock", {}).get("layout", {})
    layout_album_keys = get_layout_keys(layout_cfg.get("album", []))
    layout_track_keys = get_layout_keys(layout_cfg.get("tracks", []))
    
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
    config_path = Path("config.toml")
    if not config_path.exists():
        return 
        
    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    ext_folder = config.get("compiler", {}).get("extensions_folder")
    ext_config = config.get("compiler", {}).get("extensions", {})
    setup_registry(ext_folder, ext_config)

    A_TAGS, A_HELPERS, T_TAGS, T_HELPERS = validate_layout(config)
    
    if not library_root:
        library_root = album_root.parent 

    meta_path = album_root / "metadata.toml"
    
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

    # 1. Acquire Physical Spine
    physical_spine = scan_physical_spine(album_root, supported_exts)

    album_defaults = raw_meta.get("album", {})
    raw_tracks_source = raw_meta.get("tracks", [])
    
    inflated_tracks = []
    
    # 2. Acquire Logical Spine & Enforce Manifest Contract
    if not raw_tracks_source:
        # Auto-generation mode for new/empty albums
        if physical_spine:
            for _ in physical_spine:
                inflated_tracks.append(album_defaults.copy())
    else:
        # Strict Parity Check
        if len(raw_tracks_source) != len(physical_spine):
            raise ValueError(
                f"Manifest Mismatch in {album_root}:\n"
                f"  - Logical Tracks: {len(raw_tracks_source)}\n"
                f"  - Physical Files: {len(physical_spine)}\n"
                "The number of [[tracks]] entries must exactly match the number of audio files."
            )
            
        for t in raw_tracks_source:
            inflated_tracks.append({**album_defaults, **t})

    # 3. Deterministic Sort (User Intent)
    # Sort by DISCNUMBER then TRACKNUMBER. 
    # Python's stable sort preserves order for missing/equal keys.
    inflated_tracks.sort(key=lambda t: (
        parse_int(t.get("DISCNUMBER", "1")), 
        parse_int(t.get("TRACKNUMBER", "0"))
    ))

    # 4. Bind Spines (Ordinal Zipper)
    zip_tracks(inflated_tracks, physical_spine)
    
    # 5. Standardize Sequence
    # Re-writes TRACKNUMBER to be strictly sequential (1..N) per Disc.
    curr_disc = None
    curr_idx = 0
    for track in inflated_tracks:
        d = parse_int(track.get("DISCNUMBER", "1"))
        if d != curr_disc:
            curr_disc = d
            curr_idx = 0
        curr_idx += 1
        
        track["DISCNUMBER"] = str(d)
        track["TRACKNUMBER"] = str(curr_idx)

    # 6. Resolve Tags
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

        for key in T_TAGS:
            resolver = find_resolver(key, "TRACK_TAGS")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                final_track[key] = str(track_source.get(key, ""))

        for key in T_HELPERS:
            resolver = find_resolver(key, "TRACK_HELPERS")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                final_track[key] = ""

        if audio_obj: del audio_obj
        final_tracks.append(final_track)

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

    cover_hash = final_album.get("cover_hash")
    cover_rel_path = final_album.get("cover_path")
    
    if cover_hash and cover_rel_path and cover_rel_path != "default_cover.png":
        cache_folder_str = config.get("storage", {}).get("thumbnail_cache_folder")
        if cache_folder_str:
            cache_folder = Path(cache_folder_str).expanduser().resolve()
            dest_thumb = cache_folder / f"{cover_hash}.png"
            
            if not dest_thumb.exists():
                src_cover = album_root / cover_rel_path
                if src_cover.exists():
                    theme_cfg = config.get("theme", {})
                    generate_thumbnail(
                        src_cover, 
                        dest_thumb, 
                        size=theme_cfg.get("thumbnail_size", 200),
                        resampling=theme_cfg.get("thumbnail_resampling", "LANCZOS")
                    )

    layout_cfg = config.get("lock", {}).get("layout", {})
    write_lock(album_root, final_album, final_tracks, layout_cfg)
