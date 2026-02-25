import ast
import mutagen
from .compare import is_match

def parse_lock_value(val):
    if isinstance(val, list): return val
    if isinstance(val, str) and val.strip().startswith("[") and val.strip().endswith("]"):
        try:
            parsed = ast.literal_eval(val)
            if isinstance(parsed, list): return parsed
        except (ValueError, SyntaxError): pass
    return val

def format_value_for_display(val):
    if isinstance(val, list): return "; ".join(str(v) for v in val)
    return str(val)

def collect_changes(album_root, lock_data, harvested_map, registry):
    album_lock = lock_data.get("album", {})
    tracks = lock_data.get("tracks", [])
    
    core_album_keys = [
        "ALBUM",
        "ALBUMARTIST",
        "DATE",
        "GENRE",
        "COMMENT",
        "ORIGINAL_DATE",
        "RELEASE_DATE"
    ]
    
    core_track_keys = [
        "TITLE",
        "ARTIST",
        "TRACKNUMBER",
        "DISCNUMBER"
    ]
    
    change_log = []
    sync_plan = {}
    injection_plan = {}
    
    for track_lock in tracks:
        info = track_lock.get("info", {})
        rel_path = info.get("track_path")
        if not rel_path: continue
        abs_track_path = (album_root / rel_path).resolve()
        abs_path_str = str(abs_track_path)
        harvest_entry = harvested_map.get(abs_path_str)
        if not harvest_entry: continue
        harvest_tags = {k.lower(): v for k, v in harvest_entry.get("tags", {}).items()}
        
        mismatches = []
        
        for key in core_album_keys:
            lock_val = album_lock.get(key)
            if not lock_val: continue
            harvest_val = harvest_tags.get(key.lower(), "")
            if not is_match(key, harvest_val, lock_val, album_lock=album_lock["info"]):
                write_val = parse_lock_value(lock_val)
                if abs_track_path not in sync_plan: sync_plan[abs_track_path] = {}
                sync_plan[abs_track_path][key.upper()] = write_val
                mismatches.append((key.upper(), harvest_val, lock_val))

        for key in core_track_keys:
            lock_val = track_lock.get(key)
            if not lock_val: continue
            harvest_val = harvest_tags.get(key.lower(), "")
            if not is_match(key, harvest_val, lock_val, album_lock=album_lock["info"]):
                write_val = parse_lock_value(lock_val)
                if abs_track_path not in sync_plan: sync_plan[abs_track_path] = {}
                sync_plan[abs_track_path][key.upper()] = write_val
                mismatches.append((key.upper(), harvest_val, lock_val))

        album_tags = album_lock.get("tags", {})
        for key, meta in registry.items():
            if meta.get("level") != "album" or meta.get("sync", True) is False: continue
            lock_val = album_tags.get(key.upper())
            if not lock_val: continue
            harvest_val = harvest_tags.get(key.lower(), "")
            if not is_match(key, harvest_val, lock_val, album_lock=album_lock["info"]):
                write_val = parse_lock_value(lock_val)
                if abs_track_path not in sync_plan: sync_plan[abs_track_path] = {}
                sync_plan[abs_track_path][key.upper()] = write_val
                mismatches.append((key.upper(), harvest_val, lock_val))

        track_tags = track_lock.get("tags", {})
        for key, meta in registry.items():
            if meta.get("level") != "tracks" or meta.get("sync", True) is False: continue
            lock_val = track_tags.get(key.upper())
            if not lock_val: continue
            harvest_val = harvest_tags.get(key.lower(), "")
            if not is_match(key, harvest_val, lock_val, album_lock=album_lock["info"]):
                write_val = parse_lock_value(lock_val)
                if abs_track_path not in sync_plan: sync_plan[abs_track_path] = {}
                sync_plan[abs_track_path][key.upper()] = write_val
                mismatches.append((key.upper(), harvest_val, lock_val))

        if mismatches:
            change_log.append(f"Track: {rel_path}")
            for key, old_v, new_v in sorted(mismatches, key=lambda x: x[0]):
                v_h = format_value_for_display(old_v).strip()
                v_l = format_value_for_display(new_v).strip()
                change_log.append(f'    {key}: "{v_h}" -> "{v_l}"')
                
    return change_log, sync_plan, injection_plan

def apply_write_plan(write_plan):
    for path, tags in write_plan.items():
        try:
            f = mutagen.File(path)
            if f is None: continue
            for key, val in tags.items(): f[key] = val
            f.save()
        except Exception as e:
            print(f"Error writing to {path.name}: {e}")
