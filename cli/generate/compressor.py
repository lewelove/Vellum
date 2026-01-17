import json

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

def compress(
    raw_tracks: list, 
    tracks_layout: list = None
):
    """
    Standard Compression Logic.
    
    Decides which tags stay in the track pool and which move to the album pool.
    
    Rules:
    1. Identify keys explicitly listed in 'tracks_layout'. These are FORCED to stay in tracks.
    2. Identify keys that are identical across ALL tracks (Common Keys).
    3. If a Common Key is NOT forced, promote it to 'album_pool'.
    4. If a Common Key IS forced, keep it in 'track_pools'.
    5. All other keys (variance) stay in 'track_pools'.
    
    Returns:
        (album_pool: dict, track_pools: list)
    """
    if not raw_tracks: 
        return {}, []

    # 1. Analyze constraints
    forced_track_keys = get_layout_keys(tracks_layout) if tracks_layout else set()
    
    # 2. Identify intersection of keys (Common Keys potential candidates)
    # We only care about keys present in the first track, as a key must be in ALL tracks to be promoted.
    first_track = raw_tracks[0]
    candidate_keys = set(first_track.keys())
    
    for track in raw_tracks[1:]:
        candidate_keys &= set(track.keys())

    album_pool = {}
    keys_to_promote = []
    
    # 3. Validation of Equality
    for key in candidate_keys:
        # Check strict equality across all tracks
        is_identical = all(t[key] == first_track[key] for t in raw_tracks)
        
        if is_identical:
            # Rule 4: If forced, do not promote
            if key in forced_track_keys:
                continue
                
            # Rule 3: Promote
            keys_to_promote.append(key)
            album_pool[key] = first_track[key]

    # 4. Construct Result
    final_track_pools = []
    for track in raw_tracks:
        t_pool = track.copy()
        for k in keys_to_promote:
            if k in t_pool: 
                del t_pool[k]
        final_track_pools.append(t_pool)

    return album_pool, final_track_pools
