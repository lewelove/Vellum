import tomllib
import hashlib
from pathlib import Path
from mutagen.flac import FLAC

from cli.update.consts import ALBUM_TAGS, ALBUM_HELPERS, TRACK_TAGS, TRACK_HELPERS
from cli.update.resolver import tags, helpers
from cli.update.zipper import scan_physical_spine, zip_tracks
from cli.update.writer import write_lock

def get_resolver_func(module, key, prefix):
    """
    Registry Lookup: resolve_tag_ALBUMARTIST or resolve_helper_unix_added
    """
    func_name = f"{prefix}_{key.lower()}"
    return getattr(module, func_name, None)

def compile_album(album_root: Path, supported_exts: list, library_root: Path = None):
    # Setup context root if not provided (fallback)
    if not library_root:
        # Try to guess or just use parent of parent? 
        # Ideally passed from main, but for safety:
        library_root = album_root.parent 

    meta_path = album_root / "metadata.toml"
    
    # --- PREAMBLE: Hashing ---
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
    
    # If no [[tracks]], infer from spine count? 
    # SSOT implies [[tracks]] defines the intent. 
    # If empty, we might create generics, but let's stick to inflating what exists.
    # If 0 tracks in toml, we might have 0 tracks in lock (valid).
    # But often we want to auto-populate. Let's assume strict toml for now or simple mapping.
    if not raw_tracks_source and physical_spine:
        # Auto-create intent for every file found
        for _ in physical_spine:
            inflated_tracks.append(album_defaults.copy())
    else:
        for t in raw_tracks_source:
            # Merge album defaults into track
            inflated_tracks.append({**album_defaults, **t})

    # --- PHASE 3: DISCNUMBER & TRACKNUMBER RESOLUTION ---
    # We must resolve these BEFORE zipping to know who gets which file.
    # We write these back into the dictionary immediately.
    
    unique_discs = set()
    
    for i, track in enumerate(inflated_tracks):
        # Resolve TRACKNUMBER
        if "TRACKNUMBER" in track:
             # Sanitize
             tn = str(track["TRACKNUMBER"]).split('/')[0].strip()
             track["TRACKNUMBER"] = tn
        else:
             track["TRACKNUMBER"] = str(i + 1)
             
        # Resolve DISCNUMBER
        if "DISCNUMBER" in track:
             dn = str(track["DISCNUMBER"]).split('/')[0].strip()
             track["DISCNUMBER"] = dn
        else:
             track["DISCNUMBER"] = "1"
             
        unique_discs.add(track["DISCNUMBER"])

    # --- PHASE 4: TRACK_PATH RESOLUTION (ZIPPER) ---
    # Modifies inflated_tracks in place
    zip_tracks(inflated_tracks, physical_spine)

    # --- PHASE 5: STANDARD KEY RESOLUTION ---
    
    # A. TRACK RESOLUTION
    final_tracks = []
    
    for track_source in inflated_tracks:
        final_track = {}
        
        # Determine Physics Context
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

        # Resolve TAGS
        for key in TRACK_TAGS:
            resolver = get_resolver_func(tags, key, "resolve_tag")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                final_track[key] = str(track_source.get(key, ""))

        # Resolve HELPERS
        for key in TRACK_HELPERS:
            resolver = get_resolver_func(helpers, key, "resolve_helper")
            if resolver:
                final_track[key] = resolver(track_ctx)
            else:
                # Helper fallback? usually empty or 0
                final_track[key] = ""

        # Close handle if needed (mutagen usually handles this, but good practice to release)
        if audio_obj:
            del audio_obj
            
        final_tracks.append(final_track)

    # B. ALBUM RESOLUTION
    # Now that we have final tracks, we can calculate aggregates
    
    final_album = {}
    
    album_ctx = {
        "source": album_defaults,
        "album_root": album_root,
        "library_root": library_root,
        "metadata_toml_hash": meta_hash,
        "total_tracks_count": len(final_tracks),
        "total_discs_count": len(unique_discs),
        "all_tracks_final": final_tracks # For duration summing
    }

    # Resolve TAGS
    for key in ALBUM_TAGS:
        resolver = get_resolver_func(tags, key, "resolve_tag")
        if resolver:
            final_album[key] = resolver(album_ctx)
        else:
            final_album[key] = str(album_defaults.get(key, ""))

    # Resolve HELPERS
    for key in ALBUM_HELPERS:
        resolver = get_resolver_func(helpers, key, "resolve_helper")
        if resolver:
            final_album[key] = resolver(album_ctx)
        else:
            final_album[key] = ""

    # --- PHASE 6: COMPRESSION AND OUTPUT ---
    write_lock(album_root, final_album, final_tracks)
