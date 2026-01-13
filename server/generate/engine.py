import json
import fnmatch

def format_toml_value(value):
    return json.dumps(value, ensure_ascii=False)

def get_layout_keys(layout: list) -> set:
    """
    Traverses the layout config to find every explicitly named key or glob.
    """
    keys = set()
    for item in layout:
        if isinstance(item, str):
            if not item.startswith("#") and item != "\n":
                keys.add(item)
        elif isinstance(item, dict):
            for tags in item.values():
                for t in tags:
                    if t != "\n":
                        keys.add(t)
    return keys

def segregate_tags(raw_tracks: list, layout: list = None, greedy: bool = False):
    if not raw_tracks: return {}, []

    first_track = raw_tracks[0]
    common_keys = set(first_track.keys())
    for track in raw_tracks[1:]:
        common_keys &= set(track.keys())

    album_pool = {}
    final_common_keys = []

    for key in common_keys:
        if all(t[key] == first_track[key] for t in raw_tracks):
            if greedy:
                album_pool[key] = first_track[key]
                final_common_keys.append(key)
            else:
                if not is_key_exempt(key, layout or []):
                    album_pool[key] = first_track[key]
                    final_common_keys.append(key)

    track_pools = []
    for track in raw_tracks:
        t_pool = track.copy()
        for k in final_common_keys:
            if k in t_pool: del t_pool[k]
        track_pools.append(t_pool)

    return album_pool, track_pools

def is_key_exempt(key: str, layout: list) -> bool:
    for item in layout:
        if isinstance(item, str) and (item == key or ( "*" in item and fnmatch.fnmatch(key, item))):
            return True
        if isinstance(item, dict):
            for tags in item.values():
                if any(t == key or ("*" in t and fnmatch.fnmatch(key, t)) for t in tags):
                    return True
    return False

def render_toml_block(pool: dict, layout: list = None) -> list:
    lines = []
    keys = list(pool.keys())
    
    if layout:
        consumed = set()
        for item in layout:
            if isinstance(item, str):
                if item == "\n": lines.append("")
                elif item.startswith("#"): lines.append(item)
                else:
                    matches = sorted([k for k in keys if k not in consumed and fnmatch.fnmatch(k, item)])
                    for k in matches:
                        lines.append(f'{k} = {format_toml_value(pool[k])}')
                        consumed.add(k)
            elif isinstance(item, dict):
                for header, tags in item.items():
                    sub_matches = [k for t in tags for k in keys if k not in consumed and fnmatch.fnmatch(k, t)]
                    if sub_matches:
                        if header: lines.append(header)
                        for t_pat in tags:
                            if t_pat == "\n": lines.append("")
                            else:
                                m = sorted([k for k in keys if k not in consumed and fnmatch.fnmatch(k, t_pat)])
                                for k in m:
                                    lines.append(f'{k} = {format_toml_value(pool[k])}')
                                    consumed.add(k)
        remaining = sorted([k for k in keys if k not in consumed])
        for k in remaining: lines.append(f'{k} = {format_toml_value(pool[k])}')
    else:
        for k in sorted(keys):
            lines.append(f'{k} = {format_toml_value(pool[k])}')
            
    return lines
