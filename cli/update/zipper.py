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

    file_cursor = 0
    last_disc = -1
    last_track_num = 0

    for track in sorted_tracks:

        d = parse_int(track.get("DISCNUMBER", "1"))
        n = parse_int(track.get("TRACKNUMBER", "0"))

        if d != last_disc:
            last_disc = d
            last_track_num = 0
        
        gap = max(1, n - last_track_num)
        skip_count = gap - 1
        
        file_cursor += skip_count

        if file_cursor < len(physical_files):
            track["track_path"] = physical_files[file_cursor]
        else:
            track["track_path"] = ""

        file_cursor += 1
        
        last_track_num = n

    return sorted_tracks
