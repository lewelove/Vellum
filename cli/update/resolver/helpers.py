import os
import datetime
import xxhash
import base64
from pathlib import Path

# --- UTILS ---

def _get_audio_info(ctx, attr, default=0):
    physics = ctx.get("physics")
    if not physics:
        return default
    return physics.get(attr, default)

def _format_ms_to_time(ms):
    if not ms:
        return "0:00"
    seconds = ms // 1000
    m, s = divmod(seconds, 60)
    h, m = divmod(m, 60)
    if h > 0:
        return f"{h}:{m:02d}:{s:02d}"
    return f"{m}:{s:02d}"

# --- ALBUM HELPERS ---

def resolve_album_helper_album_root_path(ctx):
    album_root = ctx.get("album_root")
    library_root = ctx.get("library_root")
    try:
        return str(album_root.relative_to(library_root))
    except (ValueError, AttributeError):
        return ""

def resolve_album_helper_metadata_toml_hash(ctx):
    return ctx.get("metadata_toml_hash", "")

def resolve_album_helper_metadata_toml_mtime(ctx):
    return ctx.get("metadata_toml_mtime", 0)

def resolve_album_helper_unix_added(ctx):
    priority_keys = [
        "UNIX_ADDED_PRIMARY",
        "UNIX_ADDED_APPLEMUSIC",
        "UNIX_ADDED_YOUTUBE",
        "UNIX_ADDED_FOOBAR",
        "UNIX_ADDED_LOCAL",
        "UNIXTIMEAPPLE",
        "UNIXTIMEYOUTUBE",
        "UNIXTIMEFOOBAR",
    ]
    
    for key in priority_keys:
        val = ctx["source"].get(key)
        if val:
            try:
                return int(val)
            except ValueError:
                continue
                
    return 0

def resolve_album_helper_date_added(ctx):
    unix = resolve_album_helper_unix_added(ctx)
    if unix <= 0: return ""
    try:
        dt = datetime.datetime.fromtimestamp(unix)
        return dt.strftime("%B %d %Y")
    except:
        return ""

def resolve_album_helper_album_duration_time(ctx):
    total_ms = 0
    tracks = ctx.get("all_tracks_final", [])
    for t in tracks:
        val = t.get("track_duration_in_ms", 0)
        try:
            total_ms += int(val)
        except (ValueError, TypeError):
            continue
    return _format_ms_to_time(total_ms)

def resolve_album_helper_cover_path(ctx):
    album_root = ctx.get("album_root")
    priorities = ["cover.png", "cover.jpg", "folder.jpg", "folder.png", "front.jpg"]
    
    for p in priorities:
        if (album_root / p).exists():
            return p
            
    for f in album_root.iterdir():
        if f.suffix.lower() in [".jpg", ".jpeg", ".png"]:
            low_name = f.name.lower()
            if "cover" in low_name or "front" in low_name:
                return f.name
                
    return "default_cover.png"

def resolve_album_helper_cover_hash(ctx):
    rel_path = resolve_album_helper_cover_path(ctx)
    if not rel_path or rel_path == "default_cover.png":
        return ""
        
    abs_path = ctx["album_root"] / rel_path
    if not abs_path.exists():
        return ""
        
    try:
        with open(abs_path, "rb") as f:
            digest = xxhash.xxh64(f.read()).digest()
            return base64.urlsafe_b64encode(digest).decode('ascii').rstrip('=')
    except Exception:
        return ""

def resolve_album_helper_cover_byte_size(ctx):
    cp = resolve_album_helper_cover_path(ctx)
    if not cp or cp == "default_cover.png":
        return 0
    
    try:
        return os.path.getsize(ctx["album_root"] / cp)
    except OSError:
        return 0

def resolve_album_helper_cover_mtime(ctx):
    cp = resolve_album_helper_cover_path(ctx)
    if not cp or cp == "default_cover.png":
        return 0
        
    try:
        return int(os.path.getmtime(ctx["album_root"] / cp))
    except OSError:
        return 0

# --- TRACK HELPERS ---

def resolve_track_helper_track_path(ctx):
    return ctx["source"].get("track_path", "")

def resolve_track_helper_track_library_path(ctx):
    tp = ctx["source"].get("track_path")
    if not tp: return ""
    full = ctx["album_root"] / tp
    try:
        return str(full.relative_to(ctx["library_root"]))
    except ValueError:
        return str(full)

def resolve_track_helper_track_mtime(ctx):
    return _get_audio_info(ctx, "mtime", 0)

def resolve_track_helper_track_size(ctx):
    return _get_audio_info(ctx, "file_size", 0)

def resolve_track_helper_lyrics_path(ctx):
    if "lyrics_path" in ctx["source"]:
        return str(ctx["source"]["lyrics_path"])
        
    if ctx["source"].get("LYRICS"):
        return "<METADATA>"

    path = ctx.get("track_path_resolved")
    if not path: return ""
    
    lyrics_dir = ctx["album_root"] / "lyrics"
    if lyrics_dir.exists() and lyrics_dir.is_dir():
        stem = path.stem 
        for ext in [".txt", ".lrc"]:
            cand = lyrics_dir / (stem + ext)
            if cand.exists():
                try:
                    return str(cand.relative_to(ctx["album_root"]))
                except ValueError:
                    pass
    
    base = path.with_suffix("")
    for ext in [".lrc", ".txt"]:
        cand = base.with_suffix(ext)
        if cand.exists():
            try:
                return str(cand.relative_to(ctx["album_root"]))
            except ValueError:
                pass
                
    return ""

def resolve_track_helper_encoding(ctx):
    fmt = _get_audio_info(ctx, "format", "UNKNOWN")
    if fmt == "Flac": return "FLAC"
    if fmt == "Mp3": return "MP3"
    if fmt == "Opus": return "OPUS"
    return fmt.upper()

def resolve_track_helper_bits_per_sample(ctx):
    return _get_audio_info(ctx, "bit_depth", 0)

def resolve_track_helper_channels(ctx):
    return _get_audio_info(ctx, "channels", 0)

def resolve_track_helper_sample_rate(ctx):
    return _get_audio_info(ctx, "sample_rate", 0)

def resolve_track_helper_track_duration_in_samples(ctx):
    sr = resolve_track_helper_sample_rate(ctx)
    ms = resolve_track_helper_track_duration_in_ms(ctx)
    if not sr or not ms: return 0
    return int((ms / 1000) * sr)

def resolve_track_helper_track_duration_in_ms(ctx):
    return _get_audio_info(ctx, "duration_ms", 0)

def resolve_track_helper_track_duration_time(ctx):
    ms = resolve_track_helper_track_duration_in_ms(ctx)
    return _format_ms_to_time(ms)
