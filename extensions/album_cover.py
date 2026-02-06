import os
import tomllib
from pathlib import Path
from cli.update.resolver.helpers import resolve_album_helper_cover_hash

def resolve_album_helper_cover_entropy(ctx):
    """
    Calculates the byte size of the thumbnail PNG in the cache.
    """
    # 1. Reuse existing helper to get the specific hash for this album
    c_hash = resolve_album_helper_cover_hash(ctx)
    if not c_hash:
        return 0

    try:
        # 2. We must read config.toml to know WHERE the thumbnails are stored
        with open("config.toml", "rb") as f:
            config = tomllib.load(f)
        
        thumb_dir = Path(config["storage"]["thumbnail_cache_folder"]).expanduser().resolve()
        thumb_file = thumb_dir / f"{c_hash}.png"

        # 3. Perform the physical lookup
        if thumb_file.exists():
            return os.path.getsize(thumb_file)
    except Exception:
        pass
        
    return 0
