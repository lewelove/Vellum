import json
import zipfile
from datetime import datetime
from collections import Counter

def parse_listenbrainz_export(zip_path, target_year=None):
    """
    Reads ListenBrainz ZIP and aggregates listens.
    Prioritizes Album Artist if available in additional_info, 
    otherwise falls back to the primary track artist.
    """
    counts = Counter()
    with zipfile.ZipFile(zip_path, 'r') as z:
        for file_info in z.infolist():
            if file_info.filename.endswith(".jsonl"):
                with z.open(file_info) as f:
                    for line in f:
                        if not line.strip():
                            continue
                        data = json.loads(line)
                        if not _matches_year(data, target_year):
                            continue
                        
                        meta = data.get("track_metadata", {})
                        info = meta.get("additional_info", {})
                        
                        album = meta.get("release_name")
                        artist = info.get("albumartist") or meta.get("artist_name")
                        
                        if artist and album:
                            counts[(artist, album)] += 1
    
    return counts

def _matches_year(scrobble, target_year):
    if not target_year:
        return True
    ts = scrobble.get("listened_at")
    if not ts:
        return False
    try:
        return datetime.fromtimestamp(ts).year == target_year
    except (ValueError, OSError):
        return False
