import time
from mutagen.flac import FLAC
from pathlib import Path
from typing import Dict, Any, Optional

class FlacExtractor:
    @staticmethod
    def extract(file_path: Path, include_image: bool = False) -> Dict[str, Any]:
        """
        Extracts metadata from a FLAC file. 
        include_image=False (default) keeps memory usage low by skipping binary data.
        """
        try:
            audio = FLAC(file_path)
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
            return {}

        # Base pool with helpers
        # duration, sample_rate etc are captured here for "immortality"
        metadata = {
            "track_path": file_path.name,
            "date_added": int(time.time()),
            "duration": round(audio.info.length, 3),
            "sample_rate": audio.info.sample_rate,
            "bits_per_sample": audio.info.bits_per_sample,
            "channels": audio.info.channels
        }

        # Extract Mutagen tags
        if audio.tags:
            for key, val_list in audio.tags.items():
                clean_key = key.upper()
                
                if len(val_list) == 1:
                    metadata[clean_key] = val_list[0]
                else:
                    metadata[clean_key] = list(val_list)

        image_data = None
        mime_type = None
        
        # Only extract binary data if explicitly requested
        if include_image and audio.pictures:
            image_data = audio.pictures[0].data
            mime_type = audio.pictures[0].mime

        return {
            "tags": metadata,
            "image_data": image_data,
            "mime_type": mime_type
        }
