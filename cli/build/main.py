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
    
    # Destination: The Server Data Folder
    server_dest = Path("~/.mpf2k/library.json").expanduser().resolve()
    
    print(f"Building Library Artifact from {lib_root}...")
    
    # Scan for all JSON locks
    lock_files = list(lib_root.rglob("metadata.lock.json"))
    
    library_lake = []
    
    # Keys to exclude from the UI artifact to save space/noise
    EXCLUDED_KEYS = {
        "metadata_toml_hash", 
        "metadata_toml_mtime", 
        "lock_hash",
    }
    
    for lock in tqdm(lock_files, desc="Aggregating", unit="album"):
        try:
            with open(lock, "r", encoding="utf-8") as f:
                data = json.load(f)
            
            album_source = data.get("album", {})
            tracks_source = data.get("tracks", [])
            
            # 1. Resolve ID
            alb_id = album_source.get("album_root_path")
            if not alb_id:
                alb_id = str(lock.parent.relative_to(lib_root))
            
            # 2. Flatten Album Object
            clean_album = { 
                "id": alb_id 
            }
            
            for k, v in album_source.items():
                if k not in EXCLUDED_KEYS:
                    clean_album[k] = v
            
            # 3. Attach Tracks
            clean_album["tracks"] = tracks_source
            
            library_lake.append(clean_album)

        except Exception as e:
            tqdm.write(f"Error processing {lock}: {e}")

    # Sort by ID (Folder Path)
    library_lake.sort(key=lambda x: x["id"])

    print(f"Writing {len(library_lake)} albums to artifact...")
    
    server_dest.parent.mkdir(parents=True, exist_ok=True)
    with open(server_dest, "w", encoding="utf-8") as f:
        # Indented JSON for readability
        json.dump(library_lake, f, ensure_ascii=False, indent=2)
        f.write("\n")
        
    elapsed = time.time() - start_time
    print(f"Build Complete in {elapsed:.2f}s.")
