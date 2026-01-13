from mutagen.flac import FLAC
from pathlib import Path

class PhysicalExtractor:
    @staticmethod
    def get_audio_payload(file_path: Path):
        """
        Returns the mutagen audio object and raw tags.
        """
        try:
            audio = FLAC(file_path)
            tags = {}
            if audio.tags:
                for key, val_list in audio.tags.items():
                    clean_key = key.upper()
                    tags[clean_key] = val_list[0] if len(val_list) == 1 else list(val_list)
            return audio, tags
        except Exception:
            return None, {}
