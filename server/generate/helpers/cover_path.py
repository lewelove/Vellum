from pathlib import Path

def resolve(album_root: Path) -> str:
    """
    Identifies the primary cover file based on priority.
    """
    priorities = ["cover.jpg", "cover.png", "folder.jpg", "folder.png", "front.jpg"]
    
    for p in priorities:
        for f in album_root.iterdir():
            if f.name.lower() == p:
                return f.name
                
    for f in album_root.iterdir():
        if f.suffix.lower() in ['.jpg', '.jpeg', '.png']:
            if 'cover' in f.name.lower() or 'front' in f.name.lower():
                return f.name
                
    return ""
