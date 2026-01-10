from collections import defaultdict
from typing import List, Dict, Tuple, Any

def normalize_tag(value: Any) -> str:
    """
    Standardizes tag values for grouping keys.
    Lists are joined into strings. 
    Missing values (None) or empty strings return "".
    """
    if value is None:
        return ""
        
    if isinstance(value, list):
        # Join list values; usually tags are distinct strings
        return "; ".join(str(v) for v in value)
        
    s_val = str(value).strip()
    return s_val

def _parse_sort_int(value: Any) -> int:
    """
    Robust integer parser for sorting (e.g., handles "1/12", "01").
    Defaults to 0.
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

def group_tracks(tracks: List[Dict[str, Any]], keys: List[str]) -> Dict[Tuple[str, ...], List[Dict[str, Any]]]:
    """
    Buckets tracks based on a dynamic list of keys.
    """
    buckets = defaultdict(list)
    
    for track in tracks:
        # Build the composite key dynamically based on config
        group_key = tuple(
            normalize_tag(track.get(k)) for k in keys
        )
        
        buckets[group_key].append(track)
        
    return dict(buckets)

def sort_album_tracks(tracks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """
    Sorts tracks in place based on Disc -> Track -> Filename.
    """
    def sort_key(t):
        disc = _parse_sort_int(t.get("DISCNUMBER"))
        track_num = _parse_sort_int(t.get("TRACKNUMBER"))
        path = t.get("track_path_absolute", "")
        
        return (disc, track_num, path)

    return sorted(tracks, key=sort_key)
