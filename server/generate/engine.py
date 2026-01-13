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

    # Pre-calculate layout lookups for performance
    album_keys = get_layout_keys(album_layout) if album_layout else set()
    tracks_keys = get_layout_keys(tracks_layout) if tracks_layout else set()

    album_pool = {}
    final_common_keys = []
    
    for key in common_keys:
        # Check if values are identical across all tracks
        if all(t[key] == first_track[key] for t in raw_tracks):
            promote = False
            
            if greedy:
                promote = True
            else:
                if key in album_keys:
                    promote = True
                elif key in tracks_keys:
                    promote = False
                else:
                    # Default: promote common tags not explicitly bound to tracks
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
    """
    Renders a dictionary to TOML lines based on a layout list.
    Entries in 'pool' that are not in 'layout' are treated as 'appendix'.
    The appendix is inserted at the position of '*' or at the end if '*' is missing.
    """
    lines = []
    
    # 1. Partition keys
    explicit_keys = get_layout_keys(layout) if layout else set()
    appendix_keys = sorted([k for k in pool.keys() if k not in explicit_keys])
    
    appendix_consumed = False

    def emit_appendix():
        nonlocal appendix_consumed
        if appendix_consumed:
            return
        for k in appendix_keys:
            lines.append(f'{k} = {format_toml_value(pool[k])}')
        appendix_consumed = True

    # 2. If no layout, dump everything
    if not layout:
        emit_appendix()
        return lines

    # 3. Process Layout
    for item in layout:
        if isinstance(item, str):
            if item == "\n":
                lines.append("")
            elif item == "*":
                emit_appendix()
            elif item.startswith("#"):
                lines.append(item)
            elif item in pool:
                lines.append(f'{item} = {format_toml_value(pool[item])}')
        
        elif isinstance(item, dict):
            for header, tags in item.items():
                # Determine if this block has content to render
                has_content = False
                for t in tags:
                    if t == "*" and not appendix_consumed and appendix_keys:
                        has_content = True
                    elif t in pool:
                        has_content = True

                if has_content:
                    if header: lines.append(header)
                    for t in tags:
                        if t == "\n":
                            lines.append("")
                        elif t == "*":
                            emit_appendix()
                        elif t in pool:
                            lines.append(f'{t} = {format_toml_value(pool[t])}')

    # 4. Safety Flush (if * was missing)
    if not appendix_consumed:
        emit_appendix()
            
    return lines
