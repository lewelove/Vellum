import re
from pathlib import Path

def natural_sort_key(s):
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', str(s))]

def scan_physical_spine(album_root: Path, supported_exts: list) -> list:
    """
    PHASE 1: THE SPINE
    Returns list of relative paths sorted naturally.
    """
    files = []
    for ext in supported_exts:
        files.extend(album_root.rglob(f"*{ext}"))
    
    files = [f for f in files if not f.name.startswith('.')]
    rel_files = [p.relative_to(album_root) for p in files]
    rel_files.sort(key=lambda p: natural_sort_key(str(p)))
    
    return rel_files

def zip_tracks(inflated_tracks: list, physical_files: list) -> list:
    """
    PHASE 4: THE ZIP
    Matches tracks to files based on DISCNUMBER and TRACKNUMBER.
    Injects 'track_path' into the track dictionary.
    """
    
    # Map (Disc, Track) -> Track Dict Object (Reference)
    target_map = {}
    
    for t in inflated_tracks:
        d = str(t.get("DISCNUMBER", "1"))
        n = str(t.get("TRACKNUMBER", "0"))
        target_map[(d, n)] = t

    # Sort keys to iterate in order: Disc 1 Track 1, Disc 1 Track 2...
    sorted_keys = sorted(target_map.keys(), key=lambda k: (
        int(k[0]) if k[0].isdigit() else 0, 
        int(k[1]) if k[1].isdigit() else 0
    ))

    # Assign files
    file_idx = 0
    for key in sorted_keys:
        if file_idx < len(physical_files):
            # Inject the path directly
            target_map[key]["track_path"] = str(physical_files[file_idx])
            file_idx += 1
        else:
            # No file for this entry
            target_map[key]["track_path"] = ""

    return inflated_tracks
