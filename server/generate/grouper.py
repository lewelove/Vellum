import os
from pathlib import Path
from collections import defaultdict
from typing import List, Dict, Tuple, Any, Optional

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

def resolve_anchor(tracks: List[Dict[str, Any]], library_root: str) -> Tuple[Optional[Path], bool]:
    """
    Calculates the 'Anchor' (Common Folder) for a group of tracks and validates it.
    
    Returns:
        (Anchor Path object, is_valid Boolean)
    
    Validity Rules:
    1. Anchor must be within library_root.
    2. No track can be deeper than 2 subdirectories from the Anchor.
    """
    if not tracks:
        return None, False

    # 1. Calculate Lowest Common Ancestor (Anchor)
    paths = [Path(t["track_path_absolute"]).parent for t in tracks]
    
    try:
        # commonpath raises ValueError if paths are on different drives
        anchor = Path(os.path.commonpath(paths))
    except ValueError:
        return None, False

    # 2. Containment Check
    # Resolve absolute paths to handle symlinks or relative inputs safely
    abs_anchor = anchor.resolve()
    abs_lib = Path(library_root).expanduser().resolve()
    
    if not abs_anchor.is_relative_to(abs_lib):
        return abs_anchor, False

    # 3. Depth Check (Proximity)
    # Allowed: 
    #   Anchor/Track.flac (Depth 0)
    #   Anchor/CD1/Track.flac (Depth 1)
    #   Anchor/BoxSet/CD1/Track.flac (Depth 2)
    max_depth_allowed = 2
    
    for p in paths:
        try:
            rel = p.relative_to(anchor)
            # "." is depth 0. parts returns tuple of path segments.
            if rel == Path("."):
                depth = 0
            else:
                depth = len(rel.parts)
                
            if depth > max_depth_allowed:
                return anchor, False
        except ValueError:
            # Should not happen if commonpath worked, but safety first
            return anchor, False

    return anchor, True
