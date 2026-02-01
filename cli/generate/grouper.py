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

def resolve_anchor(
    tracks: List[Dict[str, Any]], 
    library_root: str, 
    supported_exts: List[str]
) -> Tuple[Optional[Path], bool]:
    """
    Calculates the 'Anchor' (Common Folder) for a group of tracks and validates it.
    
    Returns:
        (Anchor Path object, is_valid Boolean)
    
    Validity Rules (Ecological Exclusivity):
    1. Anchor must be within library_root.
    2. The group must 'own' the Anchor: No supported audio files in the Anchor 
       (or its subdirectories) can exist outside of this group.
    """
    if not tracks:
        return None, False

    # 1. Determine the Anchor (Geometric Center)
    paths = []
    group_paths_set = set()
    
    for t in tracks:
        p_str = t.get("track_path_absolute")
        if p_str:
            p = Path(p_str).resolve()
            paths.append(str(p))
            group_paths_set.add(p)
    
    if not paths:
        return None, False

    try:
        # commonpath returns the longest common sub-path.
        # If passed a single file, it returns the file path itself.
        # If passed sibling files, it returns the parent directory.
        common = os.path.commonpath(paths)
        anchor = Path(common)
    except ValueError:
        return None, False

    if anchor.is_file():
        anchor = anchor.parent

    abs_anchor = anchor.resolve()
    abs_lib = Path(library_root).expanduser().resolve()
    
    if not abs_anchor.is_relative_to(abs_lib):
        return abs_anchor, False

    # 2. Ecological Exclusivity Check
    # We must scan the calculated anchor for any "Alien" files.
    
    # Normalize extensions for case-insensitive matching
    ext_lookup = set(e.lower() for e in supported_exts)
    
    for root, _, files in os.walk(abs_anchor):
        for file in files:
            file_path = Path(root) / file
            
            if file_path.suffix.lower() in ext_lookup:
                # We found a supported audio file.
                # It MUST be in our group.
                if file_path.resolve() not in group_paths_set:
                    # Alien detected. This folder structure is contaminated.
                    return abs_anchor, False

    return abs_anchor, True
