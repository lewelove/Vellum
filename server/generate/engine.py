import json
import fnmatch

def format_toml_value(value):
    """
    Converts a Python value to a TOML-compatible string representation.
    Uses json.dumps to handle quoting and escaping securely.
    """
    return json.dumps(value, ensure_ascii=False)

def get_matching_keys(pool: dict, pattern: str, consumed: set) -> list:
    """
    Helper to find keys in the pool that match a specific pattern (exact or glob).
    Only returns keys that have NOT been consumed yet.
    """
    if "*" in pattern:
        # Glob matching
        matches = [k for k in pool.keys() if k not in consumed and fnmatch.fnmatch(k, pattern)]
        matches.sort()
        return matches
    else:
        # Exact matching
        if pattern in pool and pattern not in consumed:
            return [pattern]
    return []

def is_key_exempt(key: str, tracks_layout: list) -> bool:
    """
    Determines if a key is covered by the tracks layout.
    Handles both Strings (unconditional) and Dictionaries (conditional clusters).
    """
    for item in tracks_layout:
        
        # 1. Handle Conditional Clusters (Dictionaries)
        if isinstance(item, dict):
            for tags_list in item.values():
                for tag_pattern in tags_list:
                    if tag_pattern == "\n":
                        continue
                    # Check exact match
                    if tag_pattern == key:
                        return True
                    # Check glob match
                    if "*" in tag_pattern and fnmatch.fnmatch(key, tag_pattern):
                        return True
            continue

        # 2. Handle Strings
        if isinstance(item, str):
            if item == "\n" or item.startswith("#"):
                continue
            
            # Check exact match
            if item == key:
                return True
                
            # Check glob match
            if "*" in item and fnmatch.fnmatch(key, item):
                return True

    return False

def segregate_tags(raw_tracks: list, tracks_layout: list):
    """
    Splits tags into a common 'album_pool' and individual 'track_pools'.
    Tags present in 'tracks_layout' are NEVER moved to album_pool.
    """
    if not raw_tracks:
        return {}, []

    # 1. Identify Candidate Common Tags
    first_track = raw_tracks[0]
    common_keys = set(first_track.keys())
    
    for track in raw_tracks[1:]:
        common_keys &= set(track.keys())

    album_pool = {}
    
    # 2. Filter Common Tags
    final_common_keys = []
    for key in common_keys:
        # Exemption Rule: If it belongs in tracks layout, keep it in tracks
        if is_key_exempt(key, tracks_layout):
            continue
            
        # Value Consistency Check
        first_val = first_track[key]
        if all(t[key] == first_val for t in raw_tracks):
            album_pool[key] = first_val
            final_common_keys.append(key)

    # 3. Build Track Pools (Subtracting Album Tags)
    track_pools = []
    for track in raw_tracks:
        track_pool = track.copy()
        for k in final_common_keys:
            del track_pool[k]
        track_pools.append(track_pool)

    return album_pool, track_pools

def process_layout(pool: dict, layout: list) -> list:
    """
    Consumes tags from the pool based on the layout and returns lines for TOML.
    Supports Strings (Static) and Dictionaries (Conditional).
    """
    consumed_keys = set()
    lines = []

    # --- PASS 1: The Scripted Layout ---
    for item in layout:
        
        # A. Handle Strings (Static/Unconditional)
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

        # B. Handle Dictionaries (Conditional Clusters)
        if isinstance(item, dict):
            # Item looks like: { "# -- Header --": ["TAG1", "\n", "TAG2"] }
            for header, tags_list in item.items():
                
                # Check if this cluster has any valid tags in the pool
                has_content = any(
                    get_matching_keys(pool, p, consumed_keys) 
                    for p in tags_list if p != "\n"
                )
                
                if has_content:
                    if header: # Print header if it's not an empty string
                        lines.append(header)
                    
                    # Process sequentially to respect "\n" placement and tag order
                    for pattern in tags_list:
                        if pattern == "\n":
                            lines.append("")
                        else:
                            matches = get_matching_keys(pool, pattern, consumed_keys)
                            # matches is already sorted for globs inside get_matching_keys
                            for key in matches:
                                if key not in consumed_keys: 
                                    val = format_toml_value(pool[key])
                                    lines.append(f'{key} = {val}')
                                    consumed_keys.add(key)

    # --- PASS 2: The Appendix (Unconsumed Metadata) ---
    remaining_meta = [k for k in pool.keys() if k.isupper() and k not in consumed_keys]
    remaining_meta.sort()
    for key in remaining_meta:
        val = format_toml_value(pool[key])
        lines.append(f'{key} = {val}')
        consumed_keys.add(key)

    # --- PASS 3: The Appendix (Unconsumed Helpers) ---
    remaining_helpers = [k for k in pool.keys() if k.islower() and k not in consumed_keys]
    remaining_helpers.sort()
    for key in remaining_helpers:
        val = format_toml_value(pool[key])
        lines.append(f'{key} = {val}')
        consumed_keys.add(key)

    return lines
