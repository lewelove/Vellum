import argparse
import json
import tomllib
from pathlib import Path
from python.update.harvester import harvest_metadata
from .syncer import collect_changes, apply_write_plan

def run_write():
    parser = argparse.ArgumentParser(
        description="Vellum Write: Sync lock metadata to audio files",
        prog="vellum write"
    )
    parser.add_argument(
        "path", 
        nargs="?", 
        help="Path to folder containing albums (Defaults to library_root in config.toml)"
    )
    
    args = parser.parse_args()

    if args.path:
        target_path = Path(args.path).expanduser().resolve()
    else:
        config_path = Path("config.toml")
        if not config_path.exists():
            print("Error: No path provided and config.toml not found.")
            return
        
        with open(config_path, "rb") as f:
            config = tomllib.load(f)
        
        target_path = Path(config["storage"]["library_root"]).expanduser().resolve()

    if not target_path.exists():
        print(f"Error: Path does not exist: {target_path}")
        return

    lock_files = list(target_path.rglob("metadata.lock.json"))
    if not lock_files:
        print(f"No metadata.lock.json files found in {target_path}")
        return

    print(f"Scanning {len(lock_files)} anchored folders...")

    for lock_path in lock_files:
        album_root = lock_path.parent
        try:
            with open(lock_path, "r", encoding="utf-8") as f:
                lock_data = json.load(f)
        except Exception: 
            continue

        album_meta = lock_data.get("album", {})
        if album_meta.get("file_tag_subset_match") is True:
            continue

        harvested_map = harvest_metadata(album_root)
        if not harvested_map: 
            continue

        change_log, sync_plan, injection_plan = collect_changes(album_root, lock_data, harvested_map)

        if injection_plan:
            count = sum(len(tags) for tags in injection_plan.values())
            print(f"Injecting {count} missing tags into: {album_root.name}")
            apply_write_plan(injection_plan)

        if change_log:
            print(f"\nDiscrepancies found in: {album_root}")
            for line in change_log:
                print(line)
            
            choice = input("\nWrite overrides to audio files? Y/n: ").strip().lower()
            if choice in ["", "y", "yes"]:
                apply_write_plan(sync_plan)
                print("Overrides applied.")
            else:
                print("Overrides skipped.")
