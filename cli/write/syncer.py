import ast
import mutagen
from .compare import is_match

def parse_lock_value(val):
    if isinstance(val, list):
        return val
    if isinstance(val, str) and val.strip().startswith("[") and val.strip().endswith("]"):
        try:
            parsed = ast.literal_eval(val)
            if isinstance(parsed, list):
                return parsed
        except (ValueError, SyntaxError):
            pass
    return val

def format_value_for_display(val):
    if isinstance(val, list):
        return "; ".join(str(v) for v in val)
    return str(val)

def collect_changes(album_root, lock_data, harvested_map):
    album_lock = lock_data.get("album", {})
    tracks = lock_data.get("tracks", [])
    album_tags = {k: v for k, v in album_lock.items() if k.isupper()}
    
    change_log = []
    sync_plan = {}
    injection_plan = {}
    
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
        expected_track_state = {**album_tags, **track_lock}
        
        mismatches = []
        
        for key, lock_val in expected_track_state.items():
            if not key.isupper() or not lock_val:
                continue
            
            harvest_val = harvest_tags.get(key, "")
            if not is_match(key, harvest_val, lock_val, album_lock=album_lock):
                write_val = parse_lock_value(lock_val)
                
                # If tag is missing from file, it goes to injection (silent)
                if not harvest_val:
                    if abs_track_path not in injection_plan:
                        injection_plan[abs_track_path] = {}
                    injection_plan[abs_track_path][key] = write_val
                # If tag exists but differs, it goes to sync (prompted)
                else:
                    if abs_track_path not in sync_plan:
                        sync_plan[abs_track_path] = {}
                    sync_plan[abs_track_path][key] = write_val
                    mismatches.append((key, harvest_val, lock_val))

        if mismatches:
            change_log.append(f"Track: {rel_path_str}")
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
            for key, val in tags.items():
                f[key] = val
            f.save()
        except Exception as e:
            print(f"Error writing to {path.name}: {e}")
