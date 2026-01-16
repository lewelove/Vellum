import tomllib
import hashlib
import os
from pathlib import Path
from mutagen.flac import FLAC

from cli.update.consts import ALBUM_TAGS, ALBUM_HELPERS, TRACK_TAGS, TRACK_HELPERS
from cli.update.resolver import tags, helpers
from cli.update.zipper import scan_physical_spine, zip_tracks, parse_int
from cli.update.writer import write_lock

def get_resolver_func(module, key, prefix):
    """
    Registry Lookup: resolve_tag_ALBUMARTIST or resolve_helper_unix_added
    """
    func_name = f"{prefix}_{key.lower()}"
    return getattr(module, func_name, None)

def compile_album(album_root: Path, supported_exts: list, library_root: Path = None):
    if not library_root:
        library_root = album_root.parent 

    meta_path = album_root / "metadata.toml"
    
    # --- PREAMBLE: Hashing & Mtime ---
    # We capture mtime here to ensure the lock reflects the state at compile time
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
    
    # 1. Resolve and Defaulting
    disc_counts = {}
    
    for track in inflated_tracks:
        # Resolve Disc (Defaults to "1")
        if "DISCNUMBER" not in track:
             track["DISCNUMBER"] = "1"
        
        # Sanitize Disc
        dn_str = str(track["DISCNUMBER"]).split('/')[0].strip()
        track["DISCNUMBER"] = dn_str
        
        # Track Counter Logic (Running Index per Disc)
        current_count = disc_counts.get(dn_str, 0)
        current_count += 1
        disc_counts[dn_str] = current_count
        
        # Resolve Track (Use explicit or running index)
        if "TRACKNUMBER" in track:
             tn_str = str(track["TRACKNUMBER"]).split('/')[0].strip()
             track["TRACKNUMBER"] = tn_str
        else:
             track["TRACKNUMBER"] = str(current_count)

    # 2. Validation (No Duplicates allowed)
    seen_pairs = set()
    for track in inflated_tracks:
        d = parse_int(track.get("DISCNUMBER"))
        n = parse_int(track.get("TRACKNUMBER"))
        pair = (d, n)
        
        if pair in seen_pairs:
            raise ValueError(f"Duplicate Disc/Track found in {album_root}: Disc {d}, Track {n}")
        seen_pairs.add(pair)

    # 3. Sorting (Crucial for Zipper)
    inflated_tracks.sort(key=lambda t: (parse_int(t["DISCNUMBER"]), parse_int(t["TRACKNUMBER"])))

    # --- PHASE 4: TRACK_PATH RESOLUTION (ZIPPER) & NORMALIZATION ---
    
    # 1. The Zip (Map Files based on Gap Logic)
    zip_tracks(inflated_tracks, physical_spine)
    
    # 2. Normalization (Rewrite TRACKNUMBER to 1..N per disc)
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
            try:
                audio_obj = FLAC(t_path_abs)
            except Exception:
                pass

        track_ctx = {
            "source": track_source,
            "album_root": album_root,
            "library_root": library_root,
            "track_path_resolved": t_path_abs,
            "audio_obj": audio_obj
        }

        for key in TRACK_TAGS:
            resolver = get_resolver_func(tags, key, "resolve_tag")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                final_track[key] = str(track_source.get(key, ""))

        for key in TRACK_HELPERS:
            resolver = get_resolver_func(helpers, key, "resolve_helper")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                final_track[key] = ""

        if audio_obj:
            del audio_obj
            
        final_tracks.append(final_track)

    # B. ALBUM RESOLUTION
    final_album = {}
    
    # Recalculate unique discs based on Normalized data
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

    for key in ALBUM_TAGS:
        resolver = get_resolver_func(tags, key, "resolve_tag")
        if resolver:
            final_album[key] = resolver(album_ctx)
        else:
            final_album[key] = str(album_defaults.get(key, ""))

    for key in ALBUM_HELPERS:
        resolver = get_resolver_func(helpers, key, "resolve_helper")
        if resolver:
            final_album[key] = resolver(album_ctx)
        else:
            final_album[key] = ""

    # --- PHASE 6: COMPRESSION AND OUTPUT ---
    write_lock(album_root, final_album, final_tracks)
