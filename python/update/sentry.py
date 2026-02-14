import os
import tomllib
import hashlib
from enum import Enum
from pathlib import Path

class TrustState(Enum):
    VALID = 0
    MISSING = 1
    BROKEN_INTENT = 2
    BROKEN_PHYSICS = 3
    BROKEN_ASSETS = 4
    FORCED = 5

def get_file_hash(path: Path) -> str:
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(8192):
            sha256.update(chunk)
    return sha256.hexdigest()

def verify_trust(album_root: Path, force: bool = False) -> TrustState:
    """
    Verifies the integrity of the album folder.
    Now checks primarily against metadata.lock.json if available,
    falling back to .toml if necessary (or verifying consistency).
    """
    if force:
        return TrustState.FORCED

    meta_path = album_root / "metadata.toml"
    lock_path = album_root / "metadata.lock.json"
    
    if not lock_path.exists():
        return TrustState.MISSING

    try:
        import json
        with open(lock_path, "rb") as f:
            lock_data = json.load(f)
    except Exception:
        return TrustState.MISSING

    lock_meta_hash = lock_data.get("album", {}).get("metadata_toml_hash")
    lock_meta_mtime = lock_data.get("album", {}).get("metadata_toml_mtime", 0)
    
    current_mtime = 0
    try:
        current_mtime = int(os.path.getmtime(meta_path))
    except OSError:
        pass

    is_mtime_valid = (current_mtime != 0 and current_mtime == lock_meta_mtime)
    
    if not is_mtime_valid:
        current_meta_hash = get_file_hash(meta_path)
        if lock_meta_hash != current_meta_hash:
            return TrustState.BROKEN_INTENT

    for track in lock_data.get("tracks", []):
        track_path_str = track.get("track_path", "")
        if not track_path_str:
            continue

        track_path = album_root / track_path_str
        if not track_path.exists():
            return TrustState.BROKEN_PHYSICS
        
        cached_mtime = track.get("track_mtime", 0)
        cached_size = track.get("track_size", 0)
        
        try:
            curr_mtime = int(os.path.getmtime(track_path))
            curr_size = os.path.getsize(track_path)
        except OSError:
            return TrustState.BROKEN_PHYSICS

        if curr_mtime != cached_mtime or curr_size != cached_size:
            return TrustState.BROKEN_PHYSICS

    cover_rel = lock_data.get("album", {}).get("cover_path")
    if cover_rel and cover_rel != "default_cover.png":
        cover_path = album_root / cover_rel
        if not cover_path.exists():
            return TrustState.BROKEN_ASSETS
            
        cached_cover_size = lock_data.get("album", {}).get("cover_byte_size", 0)
        try:
            curr_cover_size = os.path.getsize(cover_path)
        except OSError:
            return TrustState.BROKEN_ASSETS
            
        if curr_cover_size != cached_cover_size:
            return TrustState.BROKEN_ASSETS

    return TrustState.VALID
