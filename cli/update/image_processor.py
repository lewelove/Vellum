from pathlib import Path
from PIL import Image

def get_resampling_method(method_name: str):
    methods = {
        "NEAREST": Image.Resampling.NEAREST,
        "BOX": Image.Resampling.BOX,
        "BILINEAR": Image.Resampling.BILINEAR,
        "HAMMING": Image.Resampling.HAMMING,
        "BICUBIC": Image.Resampling.BICUBIC,
        "LANCZOS": Image.Resampling.LANCZOS,
    }
    return methods.get(method_name.upper(), Image.Resampling.LANCZOS)

def generate_thumbnail(
    source_path: Path, 
    dest_path: Path, 
    size: int = 200, 
    resampling: str = "LANCZOS"
):
    """
    Generates a 1:1 center-cropped PNG thumbnail.
    """
    try:
        with Image.open(source_path) as img:
            # 1. Convert to RGB (handles RGBA, P, etc.)
            img = img.convert("RGB")
            
            # 2. Center Crop Logic (1:1 Aspect Ratio)
            width, height = img.size
            short_side = min(width, height)
            
            left = (width - short_side) / 2
            top = (height - short_side) / 2
            right = (width + short_side) / 2
            bottom = (height + short_side) / 2
            
            img = img.crop((left, top, right, bottom))
            
            # 3. Resize
            resample_method = get_resampling_method(resampling)
            img = img.resize((size, size), resample_method)
            
            # 4. Save as PNG (Always Lossless)
            dest_path.parent.mkdir(parents=True, exist_ok=True)
            
            # optimize=True allows PIL to do basic zlib optimization (fast)
            img.save(dest_path, "PNG", optimize=True)
            
    except Exception as e:
        print(f"Error processing thumbnail for {source_path}: {e}")
