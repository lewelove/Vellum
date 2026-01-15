import re
from pathlib import Path

def natural_sort_key(s):
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', str(s))]

def scan_physical_spine(album_root: Path, supported_exts: list) -> list:
    """
    Phase 1: Scans and returns a naturally sorted list of relative paths.
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
    Phase 4 (Logic): Matches inflated metadata tracks to physical files 
    using DISCNUMBER and TRACKNUMBER.
    """
    zipped = []
    
    # Map key -> metadata dict
    # Key is tuple (disc_str, track_str)
    track_map = {}
    
    # We assume Phase 3 has already populated/defaulted DISCNUMBER/TRACKNUMBER
    for t in inflated_tracks:
        dn = str(t.get("DISCNUMBER", "1"))
        tn = str(t.get("TRACKNUMBER", "0")) 
        key = (dn, tn)
        track_map[key] = t

    # Sort keys: Disc asc, Track asc
    def sort_k(k):
        d_int = int(k[0]) if k[0].isdigit() else 0
        t_int = int(k[1]) if k[1].isdigit() else 0
        return (d_int, t_int)

    sorted_keys = sorted(track_map.keys(), key=sort_k)
    
    # Zip against physical spine
    used_files = 0
    for key in sorted_keys:
        if used_files < len(physical_files):
            file_path = physical_files[used_files]
            meta = track_map[key]
            zipped.append({"meta": meta, "file": file_path})
            used_files += 1
            
    return zipped
