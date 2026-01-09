from mutagen.flac import FLAC
from typing import Dict, Any

class FlacExtractor:
    @staticmethod
    def extract(file_path: str) -> Dict[str, Any]:
        audio = FLAC(file_path)
        
        metadata = {
            "track_path": file_path
        }

        # Extract EVERY tag as is. 
        # In Mutagen iteration, 'val' is the string value.
        for key, val in audio.tags:
            metadata[key.upper()] = val

        image_data = None
        if audio.pictures:
            image_data = audio.pictures[0].data

        return {
            "tags": metadata,
            "image_data": image_data
        }
