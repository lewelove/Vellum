import argparse
import json
from pathlib import Path
from cli.update.harvester import harvest_metadata
from .syncer import compare_lock_to_harvest

def run_write():
    parser = argparse.ArgumentParser(description="Vellum Write: Sync lock metadata to audio files")
    parser.add_argument("path", help="Path to folder containing albums with metadata.lock.json")
    
    args = parser.parse_args()
    target_path = Path(args.path).expanduser().resolve()

    if not target_path.exists():
        print(f"Error: Path does not exist: {target_path}")
        return

    # Find all folders anchored by a lock file
    lock_files = list(target_path.rglob("metadata.lock.json"))
    
    if not lock_files:
        print("No metadata.lock.json files found in the provided path.")
        return

    print(f"Scanning {len(lock_files)} anchored folders...")

    for lock_path in lock_files:
        album_root = lock_path.parent
        
        try:
            with open(lock_path, "r", encoding="utf-8") as f:
                lock_data = json.load(f)
        except Exception as e:
            print(f"Error reading {lock_path}: {e}")
            continue

        # 1. Run harvest on this specific folder
        harvested_map = harvest_metadata(album_root)
        
        if not harvested_map:
            continue

        # 2. Compare data
        compare_lock_to_harvest(album_root, lock_data, harvested_map)
