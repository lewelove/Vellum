import os
from pathlib import Path
from typing import List, Dict, Any
from tqdm import tqdm
from .extractor import FlacExtractor

def scan_library(library_root: str, supported_extensions: List[str]) -> List[Dict[str, Any]]:
    """
    Walks the filesystem and extracts TEXT tags only.
    Uses tqdm for progress tracking and keeps image binary data out of RAM.
    """
    root_path = Path(library_root).expanduser().resolve()
    all_tracks = []
    
    print(f"Indexing library: {root_path}")

    # Pass 0: Count files for progress bar accuracy
    file_list = []
    for root, _, files in os.walk(root_path):
        for f in files:
            if any(f.lower().endswith(ext) for ext in supported_extensions):
                file_list.append(Path(root) / f)

    # Pass 1: Harvest text metadata
    for full_path in tqdm(file_list, desc="Harvesting Tags", unit="track"):
        # We explicitly set include_image=False here
        data = FlacExtractor.extract(full_path, include_image=False)
        
        if data and data.get("tags"):
            track_object = data["tags"]
            # Track absolute path is required for Pass 2 (image extraction)
            track_object["track_path_absolute"] = str(full_path)
            all_tracks.append(track_object)

    print(f"Harvested {len(all_tracks)} tracks.")
    return all_tracks
