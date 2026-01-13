import os
from pathlib import Path

def resolve(album_root: Path, cover_name: str) -> int:
    if not cover_name:
        return 0
    full_path = album_root / cover_name
    try:
        return os.path.getsize(full_path)
    except OSError:
        return 0
