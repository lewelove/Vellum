import json
import fnmatch

def format_toml_value(value):
    return json.dumps(value, ensure_ascii=False)

def is_key_exempt(key: str, tracks_layout: list) -> bool:
    """
    Checks if a key is mentioned in the tracks layout.
    Handles both strings ("TITLE") and Conditional Clusters ({ "# H": ["TAG"] }).
    """
    for item in tracks_layout:
        # Handle Conditional Cluster (Dict)
        if isinstance(item, dict):
            for tags_list in item.values():
                for tag_pattern in tags_list:
                    if tag_pattern == key or ("*" in tag_pattern and fnmatch.fnmatch(key, tag_pattern)):
                        return True
            continue

        # Handle Standard Item (String)
        if isinstance(item, str):
            if item == "\n" or item.startswith("#"):
                continue
            if item == key or ("*" in item and fnmatch.fnmatch(key, item)):
                return True

    return False

def segregate_tags(raw_tracks: list, tracks_layout: list):
    # (Same logic as before, just using the updated is_key_exempt)
    if not raw_tracks:
        return {}, []

    first_track = raw_tracks[0]
    common_keys = set(first_track.keys())
    
    for track in raw_tracks[1:]:
        common_keys &= set(track.keys())

    album_pool = {}
    final_common_keys = []
    
    for key in common_keys:
        if is_key_exempt(key, tracks_layout):
            continue
            
        first_val = first_track[key]
        if all(t[key] == first_val for t in raw_tracks):
            album_pool[key] = first_val
            final_common_keys.append(key)

    track_pools = []
    for track in raw_tracks:
        track_pool = track.copy()
        for k in final_common_keys:
            del track_pool[k]
        track_pools.append(track_pool)
        
    return album_pool, track_pools

def get_matching_keys(pool: dict, pattern: str, consumed: set) -> list:
    """Helper to find keys matching a specific pattern (exact or glob)."""
    if "*" in pattern:
        matches = [k for k in pool.keys() if k not in consumed and fnmatch.fnmatch(k, pattern)]
        matches.sort()
        return matches
    else:
        if pattern in pool and pattern not in consumed:
            return [pattern]
    return []

def process_layout(pool: dict, layout: list) -> list:
    consumed_keys = set()
    lines = []

    # --- PASS 1: The Scripted Layout ---
    for item in layout:
        
        # 1. Handle Strings (Direct Instructions)
        if isinstance(item, str):
            if item == "\n":
                lines.append("")
                continue
            
            # Unconditional Header
            if item.startswith("#"):
                lines.append(item)
                continue
            
            # Direct Tag (or Glob String)
            matches = get_matching_keys(pool, item, consumed_keys)
            for key in matches:
                val = format_toml_value(pool[key])
                lines.append(f'{key} = {val}')
                consumed_keys.add(key)
            continue

        # 2. Handle Dictionaries (Conditional Clusters)
        if isinstance(item, dict):
            # Item looks like: { "# -- Header --": ["TAG1", "TAG2"] }
            for header, tags_list in item.items():
                
                # Dry Run: Find all candidates for this cluster
                cluster_candidates = []
                for pattern in tags_list:
                    found = get_matching_keys(pool, pattern, consumed_keys)
                    cluster_candidates.extend(found)
                
                # The Decision Gate: Only print if candidates exist
                if cluster_candidates:
                    lines.append(header) # Print the conditional header
                    
                    # Sort candidates alphabetically for tidiness within the cluster
                    cluster_candidates.sort()
                    
                    for key in cluster_candidates:
                        # Double check we haven't consumed it in this very loop (edge case)
                        if key not in consumed_keys: 
                            val = format_toml_value(pool[key])
                            lines.append(f'{key} = {val}')
                            consumed_keys.add(key)

    # --- PASS 2: The Appendix (Unconsumed Metadata) ---
    remaining_meta = [k for k in pool.keys() if k.isupper() and k not in consumed_keys]
    remaining_meta.sort()
    
    if remaining_meta:
        # User requested explicit headers, but appendix needs to be separated somehow.
        # We can detect if we need a newline spacer.
        if lines and lines[-1] != "":
            lines.append("")
            
        for key in remaining_meta:
            val = format_toml_value(pool[key])
            lines.append(f'{key} = {val}')
            consumed_keys.add(key)

    # --- PASS 3: The Appendix (Unconsumed Helpers) ---
    remaining_helpers = [k for k in pool.keys() if k.islower() and k not in consumed_keys]
    remaining_helpers.sort()
    
    if remaining_helpers:
        if lines and lines[-1] != "":
            lines.append("")

        for key in remaining_helpers:
            val = format_toml_value(pool[key])
            lines.append(f'{key} = {val}')
            consumed_keys.add(key)

    return lines
