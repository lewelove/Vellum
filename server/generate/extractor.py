import time
from mutagen.flac import FLAC
from pathlib import Path
from typing import Dict, Any, Optional

class FlacExtractor:
    @staticmethod
    def extract(file_path: Path) -> Dict[str, Any]:
        try:
            audio = FLAC(file_path)
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
            return {}

        # Base pool with helpers
        # track_path keeps the filename for display purposes (layout)
        metadata = {
            "track_path": file_path.name,
            "date_added": int(time.time())
        }

        # Extract Mutagen tags
        if audio.tags:
            for key, val_list in audio.tags.items():
                clean_key = key.upper()
                
                # Logic: 1 tag = 1 string literal, >1 tags = list literal
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
