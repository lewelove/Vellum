from .compare import is_match

def compare_lock_to_harvest(album_root, lock_data, harvested_map):
    """
    Compares uppercase keys from lock_data (merging album-level tags into tracks) 
    against harvested_map.
    """
    album_lock = lock_data.get("album", {})
    tracks = lock_data.get("tracks", [])
    
    # Extract only uppercase tags from the album block to merge into tracks
    album_tags = {k: v for k, v in album_lock.items() if k.isupper()}
    
    for track_lock in tracks:
        rel_path_str = track_lock.get("track_path")
        if not rel_path_str:
            continue

        abs_track_path = (album_root / rel_path_str).resolve()
        abs_path_str = str(abs_track_path)

        harvest_entry = harvested_map.get(abs_path_str)
        if not harvest_entry:
            continue

        harvest_tags = harvest_entry.get("tags", {})
        
        # Merge: Album tags + Track-specific tags. 
        # Track-specific values take precedence if a collision occurs.
        expected_track_state = {**album_tags, **track_lock}
        
        mismatches = []
        
        # Iterate over all uppercase keys in the expanded state
        for key, lock_val in expected_track_state.items():
            if not key.isupper():
                continue
            
            harvest_val = harvest_tags.get(key, "")

            if not is_match(key, harvest_val, lock_val, album_lock=album_lock):
                mismatches.append((key, harvest_val, lock_val))

        if mismatches:
            print(f"Track: {rel_path_str}")
            for key, old_v, new_v in sorted(mismatches, key=lambda x: x[0]):
                v_h = str(old_v).strip()
                v_l = str(new_v).strip()
                print(f'    {key}: "{v_h}" -> "{v_l}"')
