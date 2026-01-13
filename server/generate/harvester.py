import os
from pathlib import Path
from typing import List, Dict, Any
from tqdm import tqdm
from .extractor import PhysicalExtractor

def scan_library(library_root: str, supported_extensions: List[str]) -> List[Dict[str, Any]]:
    """
    Legacy harvester kept for compatibility. 
    Note: The new compiler in main.py uses direct folder discovery.
    """
    root_path = Path(library_root).expanduser().resolve()
    all_tracks = []
    
    file_list = []
    for root, _, files in os.walk(root_path):
        for f in files:
            if any(f.lower().endswith(ext) for ext in supported_extensions):
                file_list.append(Path(root) / f)

    for full_path in tqdm(file_list, desc="Harvesting Tags", unit="track"):
        _, tags = PhysicalExtractor.get_audio_payload(full_path)
        if tags:
            tags["track_path_absolute"] = str(full_path)
            all_tracks.append(tags)

    return all_tracks
