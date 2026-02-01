import re
from pathlib import Path

def natural_sort_key(s):
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', str(s))]

def parse_int(val) -> int:
    """
    Safely parses integer values from strings like '1', '01', '1/12'.
    Defaults to 0 on failure.
    """
    if val is None:
        return 0
    s = str(val).strip()
    if "/" in s:
        s = s.split("/")[0]
    if s.isdigit():
        return int(s)
    return 0

def scan_physical_spine(album_root: Path, supported_exts: list) -> list:
    files = []
    for ext in supported_exts:
        files.extend(album_root.rglob(f"*{ext}"))
    
    files = [f for f in files if not f.name.startswith('.')]
    rel_files = [p.relative_to(album_root) for p in files]
    rel_files.sort(key=lambda p: natural_sort_key(str(p)))
    
    return [str(p) for p in rel_files]

def zip_tracks(sorted_tracks: list, physical_files: list) -> list:
    """
    Performs a deterministic 1:1 ordinal binding between the Logical Spine
    (sorted_tracks) and the Physical Spine (physical_files).
    
    Pre-condition: len(sorted_tracks) must equal len(physical_files).
    """
    for track, file_path in zip(sorted_tracks, physical_files):
        track["track_path"] = file_path
        
    return sorted_tracks
