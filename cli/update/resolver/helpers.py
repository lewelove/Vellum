import os
import datetime
from pathlib import Path

# --- UTILS ---

def _get_audio_info(ctx, attr, default=0):
    audio = ctx.get("audio_obj")
    if not audio or not hasattr(audio, "info"):
        return default
    return getattr(audio.info, attr, default)

# --- ALBUM SCOPE ---

def resolve_helper_album_root_path(ctx):
    album_root = ctx.get("album_root")
    library_root = ctx.get("library_root")
    try:
        return str(album_root.relative_to(library_root))
    except (ValueError, AttributeError):
        return ""

def resolve_helper_metadata_toml_hash(ctx):
    return ctx.get("metadata_toml_hash", "")

def resolve_helper_unix_added(ctx):
# This logic can be moved to ui in the future to set priority dynamically
    priority_keys = [
        "UNIX_ADDED_PRIMARY",
        "UNIX_ADDED_LOCAL",
        "UNIX_ADDED_APPLEMUSIC",
        "UNIX_ADDED_YOUTUBE",
        "UNIXTIMEAPPLE",
        "UNIXTIMEFOOBAR",
        "UNIXTIMEYOUTUBE",
    ]
    
    for key in priority_keys:
        val = ctx["source"].get(key)
        if val:
            try:
                return int(val)
            except ValueError:
                continue
                
    return 0

def resolve_helper_date_added(ctx):
# Same as in resolve_helper_unix_added
    unix = resolve_helper_unix_added(ctx)
    if unix <= 0: return ""
    try:
        dt = datetime.datetime.fromtimestamp(unix)
        return dt.strftime("%B %d %Y")
    except:
        return ""

def resolve_helper_cover_path(ctx):
    album_root = ctx.get("album_root")
    priorities = ["cover.jpg", "cover.png", "folder.jpg", "folder.png", "front.jpg"]
    
    for p in priorities:
        if (album_root / p).exists():
            return p
            
    for f in album_root.iterdir():
        if f.suffix.lower() in [".jpg", ".jpeg", ".png"]:
            low_name = f.name.lower()
            if "cover" in low_name or "front" in low_name:
                return f.name
                
    return "default_cover.png"

def resolve_helper_cover_path_absolute(ctx):
    cp = resolve_helper_cover_path(ctx)
    if not cp or cp == "default_cover.png":
        return "public/default_cover.png"
    
    full = ctx["album_root"] / cp
    try:
        return str(full.relative_to(ctx["library_root"]))
    except ValueError:
        return str(full)

def resolve_helper_cover_byte_size(ctx):
    cp = resolve_helper_cover_path(ctx)
    if not cp or cp == "default_cover.png":
        return 0
    
    try:
        return os.path.getsize(ctx["album_root"] / cp)
    except OSError:
        return 0

def resolve_helper_cover_mtime(ctx):
    cp = resolve_helper_cover_path(ctx)
    if not cp or cp == "default_cover.png":
        return 0
        
    try:
        return int(os.path.getmtime(ctx["album_root"] / cp))
    except OSError:
        return 0

# --- TRACK SCOPE ---

def resolve_helper_track_path(ctx):
    return ctx["source"].get("track_path", "")

def resolve_helper_track_path_absolute(ctx):
    tp = ctx["source"].get("track_path")
    if not tp: return ""
    full = ctx["album_root"] / tp
    try:
        return str(full.relative_to(ctx["library_root"]))
    except ValueError:
        return str(full)

def resolve_helper_track_mtime(ctx):
    path = ctx.get("track_path_resolved")
    if not path: return 0
    try:
        return int(os.path.getmtime(path))
    except OSError:
        return 0

def resolve_helper_track_size(ctx):
    path = ctx.get("track_path_resolved")
    if not path: return 0
    try:
        return os.path.getsize(path)
    except OSError:
        return 0

def resolve_helper_lyrics_path(ctx):
    path = ctx.get("track_path_resolved")
    if not path: return ""
    
    base = path.with_suffix("")
    for ext in [".lrc", ".txt"]:
        cand = base.with_suffix(ext)
        if cand.exists():
            try:
                return str(cand.relative_to(ctx["album_root"]))
            except ValueError:
                pass
    return ""

def resolve_helper_lyrics_path_absolute(ctx):
    lp = resolve_helper_lyrics_path(ctx)
    if not lp: return ""
    full = ctx["album_root"] / lp
    try:
        return str(full.relative_to(ctx["library_root"]))
    except ValueError:
        return str(full)

def resolve_helper_encoding(ctx):
    audio = ctx.get("audio_obj")
    if not audio: return "UNKNOWN"
    cls_name = audio.__class__.__name__
    if "FLAC" in cls_name: return "FLAC"
    return "UNKNOWN"

def resolve_helper_bits_per_sample(ctx):
    return _get_audio_info(ctx, "bits_per_sample", 0)

def resolve_helper_channels(ctx):
    return _get_audio_info(ctx, "channels", 0)

def resolve_helper_sample_rate(ctx):
    return _get_audio_info(ctx, "sample_rate", 0)

def resolve_helper_track_duration_in_samples(ctx):
    return _get_audio_info(ctx, "total_samples", 0)

def resolve_helper_track_duration_in_ms(ctx):
    audio = ctx.get("audio_obj")
    if not audio or not hasattr(audio, "info"):
        return 0
    length = getattr(audio.info, 'length', 0)
    return int(length * 1000)

def resolve_helper_track_duration_time(ctx):
    ms = resolve_helper_track_duration_in_ms(ctx)
    seconds = ms // 1000
    m, s = divmod(seconds, 60)
    h, m = divmod(m, 60)
    if h > 0:
        return f"{h:02d}:{m:02d}:{s:02d}"
    return f"{m:02d}:{s:02d}"
