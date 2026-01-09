import os
import time
from mutagen.flac import FLAC
from pathlib import Path
from typing import Dict, Any, List

class FlacExtractor:
    @staticmethod
    def extract(file_path: Path) -> Dict[str, Any]:
        try:
            audio = FLAC(file_path)
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
            return {}

        # Base pool with helpers (no leading underscores)
        metadata = {
            "track_path": file_path.name,
            "date_added": int(time.time())
        }

        # Extract Mutagen tags
        # audio.tags is a dict-like object where values are lists of strings
        if audio.tags:
            for key, val_list in audio.tags.items():
                clean_key = key.upper()
                
                # Logic: 1 tag = 1 string literal (unwrapped), >1 tags = list literal
                if len(val_list) == 1:
                    metadata[clean_key] = val_list[0]
                else:
                    metadata[clean_key] = list(val_list)

        image_data = None
        if audio.pictures:
            image_data = audio.pictures[0].data

        return {
            "tags": metadata,
            "image_data": image_data
        }
