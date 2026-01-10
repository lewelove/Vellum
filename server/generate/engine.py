import json
import fnmatch

def format_toml_value(value):
    """
    Converts a Python value to a TOML-compatible string representation.
    """
    return json.dumps(value, ensure_ascii=False)

def get_matching_keys(pool: dict, pattern: str, consumed: set) -> list:
    """
    Helper to find keys in the pool that match a specific pattern.
    """
    if "*" in pattern:
        matches = [k for k in pool.keys() if k not in consumed and fnmatch.fnmatch(k, pattern)]
        matches.sort()
        return matches
    else:
        if pattern in pool and pattern not in consumed:
            return [pattern]
    return []

def is_key_exempt(key: str, tracks_layout: list) -> bool:
    """
    Determines if a key is explicitly defined in the tracks layout.
    """
    for item in tracks_layout:
        if isinstance(item, dict):
            for tags_list in item.values():
                for tag_pattern in tags_list:
                    if tag_pattern == "\n": continue
                    if tag_pattern == key: return True
                    if "*" in tag_pattern and fnmatch.fnmatch(key, tag_pattern): return True
            continue

        if isinstance(item, str):
            if item == "\n" or item.startswith("#"): continue
            if item == key: return True
            if "*" in item and fnmatch.fnmatch(key, item): return True

    return False

def segregate_tags(raw_tracks: list, tracks_layout: list):
    """
    Splits tags into a common 'album_pool' and individual 'track_pools'.
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
        # Internal keys that should stay on tracks usually, 
        # unless explicit, but let's stick to layout rules.
        if key == "_embedded_image": 
            continue 

        if is_key_exempt(key, tracks_layout):
            continue
            
        first_val = first_track[key]
        if all(t[key] == first_val for t in raw_tracks):
            album_pool[key] = first_val
            final_common_keys.append(key)

    # 3. Build Track Pools
    track_pools = []
    for track in raw_tracks:
        track_pool = track.copy()
        
        # Remove embedded image from pool to avoid printing bytes to TOML
        if "_embedded_image" in track_pool:
            del track_pool["_embedded_image"]

        for k in final_common_keys:
            if k in track_pool:
                del track_pool[k]
        track_pools.append(track_pool)

    return album_pool, track_pools

def process_layout(pool: dict, layout: list) -> list:
    """
    Consumes tags from the pool based on the layout and returns TOML lines.
    """
    consumed_keys = set()
    lines = []

    # --- PASS 1: The Scripted Layout ---
    for item in layout:
        if isinstance(item, str):
            if item == "\n":
                lines.append("")
                continue
            if item.startswith("#"):
                lines.append(item)
                continue
            
            matches = get_matching_keys(pool, item, consumed_keys)
            for key in matches:
                val = format_toml_value(pool[key])
                lines.append(f'{key} = {val}')
                consumed_keys.add(key)
            continue

        if isinstance(item, dict):
            for header, tags_list in item.items():
                has_content = any(
                    get_matching_keys(pool, p, consumed_keys) 
                    for p in tags_list if p != "\n"
                )
                if has_content:
                    if header: lines.append(header)
                    for pattern in tags_list:
                        if pattern == "\n":
                            lines.append("")
                        else:
                            matches = get_matching_keys(pool, pattern, consumed_keys)
                            for key in matches:
                                if key not in consumed_keys: 
                                    val = format_toml_value(pool[key])
                                    lines.append(f'{key} = {val}')
                                    consumed_keys.add(key)

    # --- PASS 2: Appendix (Upper) ---
    remaining_meta = [k for k in pool.keys() if k.isupper() and k not in consumed_keys]
    remaining_meta.sort()
    for key in remaining_meta:
        val = format_toml_value(pool[key])
        lines.append(f'{key} = {val}')
        consumed_keys.add(key)

    # --- PASS 3: Appendix (Lower/Helpers) ---
    remaining_helpers = [k for k in pool.keys() if k.islower() and k not in consumed_keys]
    remaining_helpers.sort()
    for key in remaining_helpers:
        val = format_toml_value(pool[key])
        lines.append(f'{key} = {val}')
        consumed_keys.add(key)

    return lines
