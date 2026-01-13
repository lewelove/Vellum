from pathlib import Path

def resolve(album_root: Path) -> str:
    """
    Identifies the primary cover file based on priority.
    """
    priorities = ["cover.jpg", "cover.png", "folder.jpg", "folder.png", "front.jpg"]
    
    # 1. Check direct matches (case insensitive)
    for p in priorities:
        for f in album_root.iterdir():
            if f.name.lower() == p:
                return f.name
                
    # 2. Fallback: Any image file containing 'cover' or 'front'
    for f in album_root.iterdir():
        if f.suffix.lower() in ['.jpg', '.jpeg', '.png']:
            if 'cover' in f.name.lower() or 'front' in f.name.lower():
                return f.name
                
    return ""
