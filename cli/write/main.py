import argparse
import json
from pathlib import Path
from cli.update.harvester import harvest_metadata
from .syncer import collect_changes, apply_write_plan

def run_write():
    parser = argparse.ArgumentParser(description="Vellum Write: Sync lock metadata to audio files")
    parser.add_argument("path", help="Path to folder containing albums with metadata.lock.json")
    
    args = parser.parse_args()
    target_path = Path(args.path).expanduser().resolve()

    if not target_path.exists():
        print(f"Error: Path does not exist: {target_path}")
        return

    lock_files = list(target_path.rglob("metadata.lock.json"))
    if not lock_files:
        print("No metadata.lock.json files found.")
        return

    print(f"Scanning {len(lock_files)} anchored folders...")

    for lock_path in lock_files:
        album_root = lock_path.parent
        try:
            with open(lock_path, "r", encoding="utf-8") as f:
                lock_data = json.load(f)
        except Exception: continue

        harvested_map = harvest_metadata(album_root)
        if not harvested_map: continue

        change_log, sync_plan, injection_plan = collect_changes(album_root, lock_data, harvested_map)

        # 1. Perform silent injections (tags missing in audio but present in lock)
        if injection_plan:
            count = sum(len(tags) for tags in injection_plan.values())
            print(f"Injecting {count} missing tags into: {album_root.name}")
            apply_write_plan(injection_plan)

        # 2. Prompt for syncs (tags that exist in audio but differ from lock)
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
