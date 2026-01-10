from collections import defaultdict
from typing import List, Dict, Tuple, Any

def normalize_tag(value: Any, default: Any = None) -> Any:
    """
    Standardizes tag values for grouping keys.
    Lists are joined into strings. Empty values fallback to default.
    """
    if value is None:
        return default
        
    if isinstance(value, list):
        return "; ".join(str(v) for v in value)
        
    s_val = str(value).strip()
    if not s_val:
        return default
        
    return s_val

def _parse_sort_int(value: Any) -> int:
    """
    Robust integer parser for sorting (e.g., handles "1/12", "01").
    Defaults to 0 (Track) or 1 (Disc) depending on context, 
    but for generic sorting 0 is a safe fallback.
    """
    if value is None:
        return 0
    
    s_val = str(value).strip()
    # Handle "1/12" format
    if "/" in s_val:
        s_val = s_val.split("/")[0]
    
    try:
        return int(s_val)
    except ValueError:
        return 0

def group_tracks(tracks: List[Dict[str, Any]]) -> Dict[Tuple[str, str, str], List[Dict[str, Any]]]:
    """
    Buckets tracks based on (ALBUMARTIST, ALBUM, CUSTOM_ID).
    Strictly falls back to 'Unknown' for core identity tags.
    """
    buckets = defaultdict(list)
    
    for track in tracks:
        # Strict Fallback for Identity
        artist = normalize_tag(track.get("ALBUMARTIST"), default="Unknown")
        album = normalize_tag(track.get("ALBUM"), default="Unknown")
        
        # Loose Fallback for ID (None means no ID)
        custom_id = normalize_tag(track.get("CUSTOM_ID"), default=None)
        
        # Composite Key
        key = (artist, album, custom_id)
        buckets[key].append(track)
        
    return dict(buckets)

def sort_album_tracks(tracks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """
    Sorts tracks in place based on Disc -> Track -> Filename.
    """
    def sort_key(t):
        disc = _parse_sort_int(t.get("DISCNUMBER"))
        # Ensure Disc 0 (missing) is treated as Disc 1 for sorting stability usually, 
        # but 0 works if all are 0.
        
        track_num = _parse_sort_int(t.get("TRACKNUMBER"))
        
        # Fallback to path string if numeric tags fail
        path = t.get("track_path_absolute", "")
        
        return (disc, track_num, path)

    # Return a new sorted list
    return sorted(tracks, key=sort_key)
