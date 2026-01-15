import os
from pathlib import Path

def resolve_cover(album_root: Path) -> dict:
    priorities = ["cover.jpg", "cover.png", "folder.jpg", "folder.png", "front.jpg"]
    
    found_name = ""
    for p in priorities:
        if (album_root / p).exists():
            found_name = p
            break
            
    if not found_name:
        for f in album_root.iterdir():
            if f.suffix.lower() in ['.jpg', '.jpeg', '.png']:
                if 'cover' in f.name.lower() or 'front' in f.name.lower():
                    found_name = f.name
                    break

    out = {}
    if found_name:
        full_path = album_root / found_name
        out["cover_path"] = found_name
        out["cover_path_absolute"] = str(full_path.resolve())
        try:
            out["cover_byte_size"] = os.path.getsize(full_path)
            out["cover_mtime"] = int(os.path.getmtime(full_path))
        except OSError:
            out["cover_byte_size"] = 0
            out["cover_mtime"] = 0
    else:
        out["cover_path"] = ""
        out["cover_path_absolute"] = ""
        out["cover_byte_size"] = 0
        out["cover_mtime"] = 0
        
    return out
