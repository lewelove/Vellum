import os
import time
from pathlib import Path
from mutagen.flac import FLAC

def resolve_track_physics(album_root: Path, rel_path: Path) -> dict:
    full_path = album_root / rel_path
    out = {}
    
    out["track_path"] = str(rel_path)
    out["track_path_absolute"] = str(full_path.resolve())
    
    try:
        stat = os.stat(full_path)
        out["track_mtime"] = int(stat.st_mtime)
        out["track_size"] = stat.st_size
    except OSError:
        out["track_mtime"] = 0
        out["track_size"] = 0

    try:
        audio = FLAC(full_path)
        info = audio.info
        
        out["encoding"] = "FLAC"
        out["bits_per_sample"] = getattr(info, "bits_per_sample", 0)
        out["channels"] = getattr(info, "channels", 0)
        out["sample_rate"] = getattr(info, "sample_rate", 0)
        out["duration_in_samples"] = getattr(info, "total_samples", 0)
        
        length = getattr(info, "length", 0)
        out["duration_in_ms"] = int(length * 1000)
        
        m, s = divmod(int(length), 60)
        h, m = divmod(m, 60)
        if h > 0:
            out["duration_time"] = f"{h:02d}:{m:02d}:{s:02d}"
        else:
            out["duration_time"] = f"{m:02d}:{s:02d}"
            
    except Exception:
        out["encoding"] = "UNKNOWN"
        out["bits_per_sample"] = 0
        out["channels"] = 0
        out["sample_rate"] = 0
        out["duration_in_samples"] = 0
        out["duration_in_ms"] = 0
        out["duration_time"] = "00:00"

    out["lyrics_path"] = ""
    out["lyrics_path_absolute"] = ""
    
    candidates = [
        full_path.with_suffix(".lrc"),
        full_path.with_suffix(".txt")
    ]
    
    for c in candidates:
        if c.exists():
            out["lyrics_path"] = str(c.relative_to(album_root))
            out["lyrics_path_absolute"] = str(c.resolve())
            break
            
    return out
