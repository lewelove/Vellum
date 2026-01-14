import os
import tomllib
import time
from pathlib import Path
from typing import List, Dict, Any

from cli.generate.extractor import PhysicalExtractor
from cli.generate.engine import segregate_tags, render_toml_block
from cli.helpers import track_path, cover_path, cover_byte_size
from cli.helpers import (
    file_mtime, file_size, encoding, 
    bits_per_sample, channels, sample_rate, 
    duration_in_samples, duration_in_ms
)

def _resolve_physics(file_path: Path, relative_path: str) -> Dict[str, Any]:
    """
    Heavy I/O: Opens the file to read headers and stats.
    """
    audio, _ = PhysicalExtractor.get_audio_payload(file_path)
    if not audio:
        return None

    return {
        "track_path": relative_path,
        "file_mtime": file_mtime.resolve(file_path),
        "file_size": file_size.resolve(file_path),
        "encoding": encoding.resolve(audio),
        "bits_per_sample": bits_per_sample.resolve(audio),
        "channels": channels.resolve(audio),
        "sample_rate": sample_rate.resolve(audio),
        "duration_in_samples": duration_in_samples.resolve(audio),
        "duration_in_ms": duration_in_ms.resolve(audio),
    }

def sync_album(album_root: Path, supported_exts: List[str]) -> bool:
    """
    Synchronizes files.toml with the filesystem.
    Returns True if files.toml was updated, False otherwise.
    """
    files_toml_path = album_root / "files.toml"
    
    # --- PHASE 1: LOAD CACHE (Inflation) ---
    cached_tracks: Dict[str, Dict] = {}
    
    if files_toml_path.exists():
        try:
            with open(files_toml_path, "rb") as f:
                data = tomllib.load(f)
                
            album_defaults = data.get("album", {})
            raw_tracks = data.get("tracks", [])
            
            # Inflate: Merge [album] defaults into each [[track]]
            for t in raw_tracks:
                inflated = {**album_defaults, **t}
                # Key the cache by filename
                if "track_path" in inflated:
                    cached_tracks[inflated["track_path"]] = inflated
        except Exception:
            # If corrupt, we treat it as empty and rebuild
            cached_tracks = {}

    # --- PHASE 2: SCAN REALITY ---
    # Use natural sort logic from helpers
    current_rel_paths = track_path.resolve(album_root, supported_exts)
    
    new_track_pool = []
    has_changes = False

    # --- PHASE 3: THE TRUST CHECK ---
    for rp in current_rel_paths:
        full_path = album_root / rp
        
        # 1. Get current filesystem stats (Fast)
        curr_mtime = file_mtime.resolve(full_path)
        curr_size = file_size.resolve(full_path)
        
        # 2. Check Cache
        cached = cached_tracks.get(rp)
        
        track_data = None
        
        if cached:
            # CHECK TRUST: Path exists + Mtime match + Size match
            cached_mtime = cached.get("file_mtime", 0)
            cached_size = cached.get("file_size", 0)
            
            if curr_mtime == cached_mtime and curr_size == cached_size:
                # GREEN STATE: Trust the cache
                track_data = cached
            else:
                # YELLOW STATE: File modified
                has_changes = True
                track_data = _resolve_physics(full_path, rp)
        else:
            # RED STATE: New file found
            has_changes = True
            track_data = _resolve_physics(full_path, rp)
            
        if track_data:
            new_track_pool.append(track_data)

    # Check for deletions (Cache had files that are no longer on disk)
    if len(cached_tracks) != len(new_track_pool):
        has_changes = True

    # Check for cover art changes
    current_cover = cover_path.resolve(album_root)
    cached_cover = cached_tracks.get(next(iter(cached_tracks), {}), {}).get("cover_path", "") if cached_tracks else ""
    
    # If cover filename changed, or we need to check its size/mtime? 
    # For now, simplistic check: if filename resolved is different, update.
    # (A robust system might check cover mtime too, but let's stick to tracks for now)
    if current_cover != cached_cover:
        has_changes = True

    # --- PHASE 4: COMMIT ---
    if not has_changes and files_toml_path.exists():
        return False

    # Re-Segregate (Greedy Mode for files.toml)
    p_album_pool, p_track_pools = segregate_tags(new_track_pool, greedy=True)
    
    # Add Cover Info to Album Pool
    if current_cover:
        p_album_pool["cover_path"] = current_cover
        p_album_pool["cover_byte_size"] = cover_byte_size.resolve(album_root, current_cover)

    # Write files.toml
    with open(files_toml_path, "w", encoding="utf-8") as f:
        f.write("[album]\n")
        f.write("\n".join(render_toml_block(p_album_pool)) + "\n\n")
        for tp in p_track_pools:
            f.write("[[tracks]]\n")
            f.write("\n".join(render_toml_block(tp)) + "\n\n")

    # --- PHASE 5: SIGNAL ---
    # Touch metadata.toml to trigger DB hydration logic (external watcher)
    meta_path = album_root / "metadata.toml"
    if meta_path.exists():
        # Update mtime to now
        os.utime(meta_path, (time.time(), time.time()))

    return True
