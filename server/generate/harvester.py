import os
from pathlib import Path
from typing import List, Dict, Any
from .extractor import FlacExtractor

def scan_library(library_root: str, supported_extensions: List[str]) -> List[Dict[str, Any]]:
    """
    Walks the filesystem, extracts metadata from compatible files, 
    and injects absolute path tracking.
    """
    root_path = Path(library_root).expanduser().resolve()
    all_tracks = []
    
    print(f"Scanning library: {root_path}")

    for root, _, files in os.walk(root_path):
        for f in files:
            # Check extension match
            if not any(f.lower().endswith(ext) for ext in supported_extensions):
                continue
            
            full_path = Path(root) / f
            
            # Extract
            data = FlacExtractor.extract(full_path)
            
            if data and data.get("tags"):
                # Inject the source of truth for sorting/logic
                data["tags"]["track_path_absolute"] = str(full_path)
                
                # Keep image data attached to the track object
                track_object = data["tags"]
                track_object["_embedded_image"] = data.get("image_data")
                
                all_tracks.append(track_object)

    print(f"Harvested {len(all_tracks)} tracks.")
    return all_tracks
