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

    ext_lookup = set(e.lower() for e in supported_exts)
    
    for root, _, files in os.walk(abs_anchor):
        for file in files:
            file_path = (Path(root) / file).resolve()
            
            if file_path.suffix.lower() in ext_lookup:
                if file_path not in group_paths_set:
                    print(f"\nExclusivity Violation: {abs_anchor}")
                    print("Group members triggering anchor collision:")
                    for p in sorted(list(set(paths))):
                        print(f"  - {p}")
                    return abs_anchor, False

    return abs_anchor, True
