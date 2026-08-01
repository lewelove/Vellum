import os
import sys
import json
import base64
import mimetypes
from pathlib import Path

import xxhash
from mutagen.flac import FLAC, Picture

KEYS_TO_EMBED = [
    "album", "albumartist", "date", "genre", "comment",
    "title", "artist", "discogs_url", "musicbrainz_url",
    "replaygain_track_gain", "replaygain_album_gain"
]
AUTO_DELETE = False
AUTO_CONVERT_TRACKNUMBER = False
AUTO_CONVERT_DISCNUMBER = False
AUTO_COVER_EMBED = False

def get_hash(data):
    h = xxhash.xxh64(data).digest()
    return base64.urlsafe_b64encode(h).decode("ascii").rstrip("=")

def is_zero_pad_conv(tag, old_val, new_val):
    u_tag = tag.upper()
    if u_tag == "TRACKNUMBER" and not AUTO_CONVERT_TRACKNUMBER:
        return False
    if u_tag == "DISCNUMBER" and not AUTO_CONVERT_DISCNUMBER:
        return False
    if u_tag not in ("TRACKNUMBER", "DISCNUMBER"):
        return False
    s_old, s_new = str(old_val), str(new_val)
    if not s_old.isdigit() or not s_new.isdigit():
        return False
    return (s_old.lstrip('0') or "0") == s_new and s_old != s_new

def render_progress_bar(current, total, width=30):
    pct = current / total if total > 0 else 1.0
    filled = int(round(width * pct))
    bar = "█" * filled + "░" * (width - filled)
    return f"\033[1;34m[{bar}]\033[0m \033[1m{current}/{total}\033[0m (\033[36m{int(pct * 100)}%\033[0m)"

def process_album(album_lock, target_dir, auto_apply):
    album_obj = album_lock.get("album", {})
    tracks_data = album_lock.get("tracks", [])
    if not tracks_data:
        return 0, False

    exclude_keys = {"info", "tags", "keys", "covers", "manifests", "file", "id"}
    album_pool = {k.lower(): v for k, v in album_obj.items() if k not in exclude_keys and not isinstance(v, dict)}
    for pool_key in ["tags", "keys"]:
        if pool_key in album_obj:
            album_pool.update({k.lower(): v for k, v in album_obj[pool_key].items()})

    total_discs = int(album_obj.get("info", {}).get("total_discs", 1))
    
    cover_filename = album_obj.get("covers", {}).get("main", {}).get("file", {}).get("path", "cover.png")
    cover_path = target_dir / cover_filename
    
    disk_cover_data = None
    disk_cover_hash = None
    if cover_path.exists():
        with open(cover_path, "rb") as cf:
            disk_cover_data = cf.read()
        disk_cover_hash = get_hash(disk_cover_data)

    first_track_path = target_dir / tracks_data[0].get("file", {}).get("path", "")
    if not first_track_path.exists():
        return 0, False
        
    first_audio = FLAC(first_track_path)
    embedded_cover_hash = None
    if first_audio.pictures:
        for pic in first_audio.pictures:
            if pic.type == 3:
                embedded_cover_hash = get_hash(pic.data)
                break

    update_cover = (disk_cover_hash is not None) and (embedded_cover_hash != disk_cover_hash)
    requires_prompt = False
    if update_cover and not AUTO_COVER_EMBED:
        requires_prompt = True

    has_actual_changes = update_cover

    tasks = []
    for t in tracks_data:
        t_file = t.get("file", {})
        rel_path = t_file.get("path")
        if not rel_path:
            continue

        target_tags = {}
        track_pool = {k.lower(): v for k, v in t.items() if k not in exclude_keys and not isinstance(v, dict)}
        for pool_key in ["tags", "keys"]:
            if pool_key in t:
                track_pool.update({k.lower(): v for k, v in t[pool_key].items()})

        for tag_name in KEYS_TO_EMBED:
            tag_key = tag_name.lower()
            val = track_pool.get(tag_key, album_pool.get(tag_key))
            if val is not None:
                target_tags[tag_name.upper()] = "; ".join(val) if isinstance(val, list) else str(val)

        target_tags["ARTIST"] = str(track_pool.get("artist", ""))
        target_tags["TITLE"] = str(track_pool.get("title", ""))
        target_tags["TRACKNUMBER"] = str(track_pool.get("tracknumber", "0"))
        
        if total_discs > 1:
            target_tags["DISCNUMBER"] = str(track_pool.get("discnumber", "1"))
    
        tasks.append({
            "path": rel_path,
            "target_tags": target_tags,
            "diffs": []
        })

    for task in tasks:
        track_file = target_dir / task["path"]
        if not track_file.exists():
            continue
            
        audio = FLAC(track_file)
        target_keys = set(task["target_tags"].keys())
        diffs = []
        
        for old_tag in list(audio.keys()):
            u_old = old_tag.upper()
            if u_old not in target_keys:
                has_actual_changes = True
                old_vals = audio.get(old_tag, [])
                old_val = "; ".join(old_vals) if isinstance(old_vals, list) else str(old_vals)
                if not AUTO_DELETE:
                    diffs.append(f"\033[31m- {u_old}: \033[90m{old_val}\033[0m")
                    requires_prompt = True

        for tag, new_val in task["target_tags"].items():
            old_vals = audio.get(tag, audio.get(tag.lower(), []))
            old_val = old_vals[0] if old_vals else ""
            if str(old_val) != str(new_val):
                has_actual_changes = True
                if not old_val:
                    diffs.append(f"\033[32m+ {tag}: \033[90m{new_val}\033[0m")
                    requires_prompt = True
                else:
                    if is_zero_pad_conv(tag, old_val, new_val):
                        pass
                    else:
                        diffs.append(f"\033[34m~ {tag}: \033[90m{old_val} -> {new_val}\033[0m")
                        requires_prompt = True

        task["diffs"] = diffs

    active_tasks = [t for t in tasks if t["diffs"]]
    common_diffs = []
    if len(active_tasks) > 1:
        first_diffs = active_tasks[0]["diffs"]
        for d in first_diffs:
            if all(d in t["diffs"] for t in active_tasks):
                common_diffs.append(d)
                
        for t in active_tasks:
            t["diffs"] = [d for d in t["diffs"] if d not in common_diffs]

    if not has_actual_changes:
        return 0, False

    show_cover_msg = update_cover and not AUTO_COVER_EMBED
    has_visible_diffs = show_cover_msg or bool(common_diffs) or bool(active_tasks)

    lines_printed = 0

    if has_visible_diffs:
        print()
        lines_printed += 1

        album_artist = album_obj.get("albumartist", "")
        album_title = album_obj.get("album", "")
        header = f"{album_artist} - {album_title}" if album_artist and album_title else target_dir.name
        print(f"\033[1;36m{header}\033[0m")
        lines_printed += 1

        if show_cover_msg:
            print()
            print("\033[33m🖼️  Cover update required\033[0m")
            lines_printed += 2

        if common_diffs or any(t["diffs"] for t in active_tasks):
            print()
            lines_printed += 1

        if common_diffs:
            print("\033[1;34m💿 Album Diff\033[0m")
            lines_printed += 1
            for d in common_diffs:
                print(f"   {d}")
                lines_printed += 1

        for task in tasks:
            if task["diffs"]:
                print(f"\033[1m🎵 {task['path']}\033[0m")
                lines_printed += 1
                for d in task["diffs"]:
                    print(f"   {d}")
                    lines_printed += 1

    if not auto_apply and requires_prompt:
        try:
            sys.stdout.write(f"\n\033[1;35mApply changes? [y/N]: \033[0m")
            sys.stdout.flush()
            lines_printed += 2
            with open('/dev/tty', 'r') as tty:
                ans = tty.readline().strip().lower()
            if ans not in ('y', 'yes'):
                return lines_printed, True
        except Exception:
            return lines_printed, True

    new_pic = None
    if update_cover:
        new_pic = Picture()
        new_pic.data = disk_cover_data
        new_pic.type = 3
        new_pic.mime = mimetypes.guess_type(cover_path)[0] or "image/jpeg"
        new_pic.desc = "Front Cover"

    for task in tasks:
        rel_path = task["path"]
        audio = FLAC(target_dir / rel_path)
        
        target_tags = task["target_tags"]
        for old_tag in list(audio.keys()):
            if old_tag.upper() not in target_tags:
                del audio[old_tag]
        for tag, val in target_tags.items():
            audio[tag] = [val]
        
        if update_cover:
            audio.clear_pictures()
            audio.add_picture(new_pic)
            
        audio.save()
        
    print("\033[32m✔ Done.\033[0m")
    lines_printed += 1
    return lines_printed, True


def main():
    try:
        data = json.load(sys.stdin)
    except Exception as e:
        print(f"Error reading JSON from stdin: {e}")
        sys.exit(1)

    raw_albums = data.get("albums", [])
    vellum_cfg = data.get("config", {}).get("vellum", {})
    action_cfg = data.get("config", {}).get("action", {})
    options_str = data.get("options", "")

    auto_apply = "--auto" in options_str or "-y" in options_str

    if "keys_to_embed" in action_cfg:
        global KEYS_TO_EMBED
        KEYS_TO_EMBED = action_cfg["keys_to_embed"]

    if "auto_delete" in action_cfg:
        global AUTO_DELETE
        AUTO_DELETE = bool(action_cfg["auto_delete"])

    if "auto_convert_tracknumber" in action_cfg:
        global AUTO_CONVERT_TRACKNUMBER
        AUTO_CONVERT_TRACKNUMBER = bool(action_cfg["auto_convert_tracknumber"])

    if "auto_convert_discnumber" in action_cfg:
        global AUTO_CONVERT_DISCNUMBER
        AUTO_CONVERT_DISCNUMBER = bool(action_cfg["auto_convert_discnumber"])

    if "auto_cover_embed" in action_cfg:
        global AUTO_COVER_EMBED
        AUTO_COVER_EMBED = bool(action_cfg["auto_cover_embed"])

    albums = []
    for album_lock in raw_albums:
        album_obj = album_lock.get("album", {})
        if album_obj.get("keys", {}).get("virtual") or album_obj.get("info", {}).get("virtual"):
            continue
        albums.append(album_lock)

    library_str = vellum_cfg.get("storage", {}).get("library", "")
    if not library_str:
        print("Error: library not defined in config")
        sys.exit(1)

    library = Path(library_str).expanduser().resolve()
    total_albums = len(albums)

    for idx, album_lock in enumerate(albums, 1):
        album_id = album_lock.get("album", {}).get("id", "")
        if not album_id:
            continue

        print(f"\n{render_progress_bar(idx, total_albums)}")
        target_dir = library / album_id
        lines_printed, was_processed = process_album(album_lock, target_dir, auto_apply)

        sys.stdout.write(f"\033[{lines_printed + 2}A\033[J")
        sys.stdout.flush()

    if total_albums > 0:
        print(f"\n{render_progress_bar(total_albums, total_albums)}")

if __name__ == "__main__":
    main()
