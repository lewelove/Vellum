import json
import tomllib
import time
from pathlib import Path
from tqdm import tqdm

def run_build():
    start_time = time.time()
    config_path = Path("config.toml")
    
    if not config_path.exists():
        print("Config not found")
        return

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    
    # Destination 1: The UI Public Folder (for Dev/Local)
    ui_dest = Path("ui/public/library.json").resolve()
    
    # Destination 2: The Server Data Folder (for Production/Backend)
    server_dest = Path("~/.mpf2k/library.json").expanduser().resolve()
    
    print(f"Building Library Artifact from {lib_root}...")
    
    # Scan for all JSON locks
    lock_files = list(lib_root.rglob("metadata.lock.json"))
    
    flat_lake = []
    
    for lock in tqdm(lock_files, desc="Aggregating", unit="album"):
        try:
            with open(lock, "r", encoding="utf-8") as f:
                data = json.load(f)
                
            album_info = data.get("album", {})
            tracks = data.get("tracks", [])
            
            # Identify the Album ID (Relative Path)
            # We rely on the album_root_path helper computed by compiler
            alb_id = album_info.get("album_root_path")
            
            # Fallback if helper missing
            if not alb_id:
                alb_id = str(lock.parent.relative_to(lib_root))
            
            # Normalize Album Info for efficient repetition
            # We create a "Track Object" that contains everything needed for the UI
            for track in tracks:
                # Merge Album Tags into Track for Flat Lake Structure
                # (The UI handles grouping, so redundant data is fine for RAM speed)
                flat_track = {
                    **album_info, # Base tags
                    **track,      # Track specifics override album
                    "album_id": alb_id,
                    "id": track.get("track_library_path") # Unique ID
                }
                
                # Cleanup internal compiler keys to save RAM
                keys_to_remove = ["metadata_toml_hash", "metadata_toml_mtime", "lock_hash"]
                for k in keys_to_remove:
                    flat_track.pop(k, None)
                    
                flat_lake.append(flat_track)
                
        except Exception as e:
            tqdm.write(f"Error processing {lock}: {e}")

    # Write Artifacts
    print(f"Writing {len(flat_lake)} tracks to artifacts...")
    
    # 1. UI
    ui_dest.parent.mkdir(parents=True, exist_ok=True)
    with open(ui_dest, "w", encoding="utf-8") as f:
        json.dump(flat_lake, f, ensure_ascii=False)
        
    # 2. Server
    server_dest.parent.mkdir(parents=True, exist_ok=True)
    with open(server_dest, "w", encoding="utf-8") as f:
        json.dump(flat_lake, f, ensure_ascii=False)
        
    elapsed = time.time() - start_time
    print(f"Build Complete in {elapsed:.2f}s.")
