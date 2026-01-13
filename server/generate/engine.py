import json

def format_toml_value(value):
    return json.dumps(value, ensure_ascii=False)

def get_layout_keys(layout: list) -> set:
    """
    Traverses the layout config to find every explicitly named key.
    """
    keys = set()
    if not layout:
        return keys
    for item in layout:
        if isinstance(item, str):
            if item != "*" and not item.startswith("#") and item != "\n":
                keys.add(item)
        elif isinstance(item, dict):
            for tags in item.values():
                for t in tags:
                    if t != "*" and t != "\n":
                        keys.add(t)
    return keys

def is_key_in_layout(key: str, layout: list) -> bool:
    """
    Checks if a key is explicitly mentioned in the layout.
    """
    for item in layout:
        if isinstance(item, str) and item == key:
            return True
        if isinstance(item, dict):
            for tags in item.values():
                if any(t == key for t in tags):
                    return True
    return False

def segregate_tags(
    raw_tracks: list, 
    album_layout: list = None, 
    tracks_layout: list = None, 
    greedy: bool = False
):
    """
    Decides which tags stay in the track pool and which move to the album pool.
    """
    if not raw_tracks: return {}, []

    first_track = raw_tracks[0]
    common_keys = set(first_track.keys())
    for track in raw_tracks[1:]:
        common_keys &= set(track.keys())

    album_pool = {}
    final_common_keys = []
    
    for key in common_keys:
        if all(t[key] == first_track[key] for t in raw_tracks):
            promote = False
            
            if greedy:
                promote = True
            else:
                in_album_cfg = is_key_in_layout(key, album_layout or [])
                in_tracks_cfg = is_key_in_layout(key, tracks_layout or [])
                
                if in_album_cfg:
                    promote = True
                elif in_tracks_cfg:
                    promote = False
                else:
                    promote = True

            if promote:
                album_pool[key] = first_track[key]
                final_common_keys.append(key)

    track_pools = []
    for track in raw_tracks:
        t_pool = track.copy()
        for k in final_common_keys:
            if k in t_pool: del t_pool[k]
        track_pools.append(t_pool)

    return album_pool, track_pools

def render_toml_block(pool: dict, layout: list = None) -> list:
    lines = []
    pool_keys = set(pool.keys())
    consumed = set()
    
    reserved = get_layout_keys(layout) if layout else set()

    def remaining_appendix(force=False):
        """
        Prints tags not yet consumed. 
        If force=False (at a '*' middle-flush), skips keys reserved for later.
        If force=True (at the end), prints everything remaining.
        """
        remaining = [k for k in pool_keys if k not in consumed]
        if not force:
            remaining = [k for k in remaining if k not in reserved]
            
        for k in sorted(remaining):
            lines.append(f'{k} = {format_toml_value(pool[k])}')
            consumed.add(k)

    if layout:
        for item in layout:
            if isinstance(item, str):
                if item == "\n": 
                    lines.append("")
                elif item == "*":
                    remaining_appendix(force=False)
                elif item.startswith("#"): 
                    lines.append(item)
                elif item in pool:
                    lines.append(f'{item} = {format_toml_value(pool[item])}')
                    consumed.add(item)
            elif isinstance(item, dict):
                for header, tags in item.items():
                    if any(t in pool or t == "*" for t in tags):
                        if header: lines.append(header)
                        for t in tags:
                            if t == "\n": 
                                lines.append("")
                            elif t == "*":
                                remaining_appendix(force=False)
                            elif t in pool:
                                lines.append(f'{t} = {format_toml_value(pool[t])}')
                                consumed.add(t)
        
        remaining_appendix(force=True)
    else:
        remaining_appendix(force=True)
            
    return lines
