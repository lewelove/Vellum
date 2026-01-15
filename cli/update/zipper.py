import re
from pathlib import Path

def natural_sort_key(s):
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', str(s))]

def scan_physical_spine(album_root: Path, supported_exts: list) -> list:
    files = []
    for ext in supported_exts:
        files.extend(album_root.rglob(f"*{ext}"))
    
    files = [f for f in files if not f.name.startswith('.')]
    
    rel_files = [p.relative_to(album_root) for p in files]
    rel_files.sort(key=lambda p: natural_sort_key(str(p)))
    
    return rel_files

def zip_tracks(inflated_tracks: list, physical_files: list) -> list:
    zipped = []
    
    track_map = {}
    for i, t in enumerate(inflated_tracks):
        tn_val = t.get("TRACKNUMBER")
        dn_val = t.get("DISCNUMBER", "1")
        
        if tn_val is not None:
            key = (str(dn_val), str(tn_val))
            track_map[key] = t
        else:
            fallback_tn = str(i + 1)
            key = (str(dn_val), fallback_tn)
            track_map[key] = t

    used_files = 0
    
    sorted_keys = sorted(track_map.keys(), key=lambda k: (int(k[0]) if k[0].isdigit() else 0, int(k[1]) if k[1].isdigit() else 0))
    
    for key in sorted_keys:
        if used_files < len(physical_files):
            file_path = physical_files[used_files]
            meta = track_map[key]
            zipped.append({"meta": meta, "file": file_path})
            used_files += 1
            
    return zipped
