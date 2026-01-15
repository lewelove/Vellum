import os
import re
import datetime
from pathlib import Path
from mutagen.flac import FLAC
from mutagen import MutagenError

# album_root_path
def resolve_helper_album_root_path(album_root: Path, library_root: Path) -> str:
    try:
        return str(album_root.relative_to(library_root))
    except ValueError:
        return ""

# unix_added
def resolve_helper_unix_added(source: dict) -> int:
    primary = source.get("UNIX_ADDED_PRIMARY")
    if primary and str(primary).strip():
        try:
            return int(primary)
        except ValueError:
            pass

    candidates = []
    for key in ["UNIX_ADDED_LOCAL", "UNIX_ADDED_APPLEMUSIC", "UNIX_ADDED_YOUTUBE"]:
        val = source.get(key)
        if val:
            try:
                candidates.append(int(val))
            except (ValueError, TypeError):
                continue

    return max(candidates) if candidates else 0

# date_added
def resolve_helper_date_added(unix_added: int) -> str:
    if unix_added <= 0:
        return ""
    try:
        dt = datetime.datetime.fromtimestamp(unix_added)
        return dt.strftime("%B %d %Y")
    except (ValueError, OSError, OverflowError):
        return ""

# cover_path
def resolve_helper_cover_path(album_root: Path) -> str:
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

# cover_path_absolute
def resolve_helper_cover_path_absolute(cover_path: str, album_root: Path, library_root: Path) -> str:
    if cover_path == "default_cover.png":
        return "public/default_cover.png"

    full_path = album_root / cover_path
    try:
        return str(full_path.relative_to(library_root))
    except ValueError:
        return str(full_path)

# cover_byte_size
def resolve_helper_cover_byte_size(album_root: Path, cover_path: str) -> int:
    if cover_path == "default_cover.png":
        return 0
        
    path = album_root / cover_path
    try:
        if path.exists():
            return path.stat().st_size
    except OSError:
        pass
    return 0
