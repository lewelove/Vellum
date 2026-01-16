import tomllib
import hashlib
import os
from enum import Enum
from pathlib import Path

class TrustState(Enum):
    VALID = 0
    MISSING = 1
    BROKEN_INTENT = 2
    BROKEN_PHYSICS = 3
    BROKEN_ASSETS = 4
    FORCED = 5  # New state for explicit force

def get_file_hash(path: Path) -> str:
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(8192):
            sha256.update(chunk)
    return sha256.hexdigest()

def verify_trust(album_root: Path, force: bool = False) -> TrustState:
    # 1. Immediate "Force Rebuild" check
    # If the user demands a force update (e.g. Python logic changed),
    # we discard the lock validity immediately.
    if force:
        return TrustState.FORCED

    meta_path = album_root / "metadata.toml"
    lock_path = album_root / "metadata.lock"
    
    if not lock_path.exists():
        return TrustState.MISSING

    try:
        with open(lock_path, "rb") as f:
            lock_data = tomllib.load(f)
    except Exception:
        return TrustState.MISSING

    # --- METADATA INTENT CHECK ---
    lock_meta_hash = lock_data.get("album", {}).get("metadata_toml_hash")
    lock_meta_mtime = lock_data.get("album", {}).get("metadata_toml_mtime", 0)
    
    current_mtime = 0
    try:
        current_mtime = int(os.path.getmtime(meta_path))
    except OSError:
        pass

    # Optimization: Trust mtime if it matches exactly
    is_mtime_valid = (current_mtime != 0 and current_mtime == lock_meta_mtime)
    
    if not is_mtime_valid:
        # Fallback: Verify Hash if mtime changed (or is 0)
        current_meta_hash = get_file_hash(meta_path)
        if lock_meta_hash != current_meta_hash:
            return TrustState.BROKEN_INTENT

    # --- PHYSICS CHECK ---
    for track in lock_data.get("tracks", []):
        track_path = album_root / track.get("track_path", "")
        # If track_path is empty (gap in zipper), skip
        if not track.get("track_path"):
            continue

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

    # --- ASSETS CHECK ---
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
