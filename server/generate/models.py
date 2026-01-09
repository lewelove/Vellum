from dataclasses import dataclass, field
from typing import List, Dict, Optional

@dataclass
class TrackMetadata:
    path: str
    title: str
    artist: str
    album: str
    album_artist: str
    track_number: str
    disc_number: str
    date: str
    genre: str
    # Raw image data from the first track found
    image_data: Optional[bytes] = None

@dataclass
class AlbumGroup:
    album_title: str
    album_artist: str
    tracks: List[TrackMetadata] = field(default_factory=list)
    
    @property
    def cover_data(self) -> Optional[bytes]:
        # Simple logic: return image data from the first track that has it
        for track in self.tracks:
            if track.image_data:
                return track.image_data
        return None
