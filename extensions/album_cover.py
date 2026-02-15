import os
import tomllib
import numpy as np
from pathlib import Path
from PIL import Image
from python.update.resolver.helpers import resolve_album_helper_cover_hash

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
    """
    Calculates the Hasler-Susstrunk Perceptual Colorfulness Metric.
    
    This is the industry standard statistical approach for measuring 
    colorfulness in natural images without requiring a reference image.
    
    Source: Hasler, D., & Susstrunk, S. (2003). "Measuring colorfulness 
    in natural images". Human Vision and Electronic Imaging VIII.
    """
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
            # Step 1: Normalize input to RGB space
            img = img.convert("RGB")
            arr = np.array(img).astype(float)
            
            r, g, b = arr[:, :, 0], arr[:, :, 1], arr[:, :, 2]
            
            # Step 2: Transform to Opponent Color Space
            # Red-Green axis
            rg = r - g
            # Yellow-Blue axis
            yb = 0.5 * (r + g) - b
            
            # Step 3: Compute Statistical Moments
            std_rg = np.std(rg)
            std_yb = np.std(yb)
            
            mean_rg = np.mean(rg)
            mean_yb = np.mean(yb)
            
            # Step 4: Aggregate Axis Statistics
            # Standard Deviation of the opponent space
            std_root = np.sqrt(std_rg**2 + std_yb**2)
            # Mean of the opponent space
            mean_root = np.sqrt(mean_rg**2 + mean_yb**2)
            
            # Step 5: Calculate Final Perceptual Score
            # Result represents the "Distance from Neutral" weighted by Variance.
            return float(std_root + (0.3 * mean_root))
            
    except Exception:
        pass
        
    return 0.0
