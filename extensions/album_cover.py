import os
import tomllib
import numpy as np
from pathlib import Path
from PIL import Image
from cli.update.resolver.helpers import resolve_album_helper_cover_hash

def resolve_album_helper_cover_entropy(ctx):
    c_hash = resolve_album_helper_cover_hash(ctx)
    if not c_hash:
        return 0

    try:
        with open("config.toml", "rb") as f:
            config = tomllib.load(f)
        
        thumb_dir = Path(config["storage"]["thumbnail_cache_folder"]).expanduser().resolve()
        thumb_file = thumb_dir / f"{c_hash}.png"

        if thumb_file.exists():
            return os.path.getsize(thumb_file)
    except Exception:
        pass
        
    return 0

def resolve_album_helper_cover_chroma(ctx):
    c_hash = resolve_album_helper_cover_hash(ctx)
    if not c_hash:
        return 0.0

    try:
        with open("config.toml", "rb") as f:
            config = tomllib.load(f)
        
        thumb_dir = Path(config["storage"]["thumbnail_cache_folder"]).expanduser().resolve()
        thumb_file = thumb_dir / f"{c_hash}.png"

        if not thumb_file.exists():
            return 0.0

        with Image.open(thumb_file) as img:
            lab = img.convert("LAB")
            arr = np.array(lab)
            
            a = arr[:, :, 1].view(np.int8).astype(np.float32)
            b = arr[:, :, 2].view(np.int8).astype(np.float32)
            
            chroma = np.sqrt(a**2 + b**2)
            return float(np.mean(chroma))
            
    except Exception:
        pass
        
    return 0.0
