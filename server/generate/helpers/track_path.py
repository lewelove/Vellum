import re
from pathlib import Path

def natural_sort_key(s):
    """
    Key for natural sorting (e.g., '2.flac' before '10.flac').
    """
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', str(s))]

def resolve(album_root: Path, supported_extensions: list) -> list:
    """
    Returns a list of relative paths to audio files, naturally sorted.
    This defines the 'Slots' for the Zipper Logic.
    """
    files = []
    for ext in supported_extensions:
        files.extend(album_root.rglob(f"*{ext}"))
    
    files = [f for f in files if not f.name.startswith('.')]
    
    files.sort(key=lambda p: natural_sort_key(p.relative_to(album_root)))
    
    return [str(p.relative_to(album_root)) for p in files]
