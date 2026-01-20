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
            # The compiler calculates 'album_root_path', use it or derive relative path
            alb_id = album_source.get("album_root_path")
            if not alb_id:
                alb_id = str(lock.parent.relative_to(lib_root))
            
            # 2. Flatten Album Object
            # Instead of { album: {...}, tracks: [...] }, we create { ...album_tags, id: "...", tracks: [...] }
            # This reduces nesting depth for the UI while keeping the logical grouping.
            clean_album = { 
                "id": alb_id 
            }
            
            # Merge source tags, excluding compiler internals
            for k, v in album_source.items():
                if k not in EXCLUDED_KEYS:
                    clean_album[k] = v
            
            # 3. Attach Tracks
            clean_album["tracks"] = tracks_source
            
            library_lake.append(clean_album)

        except Exception as e:
            tqdm.write(f"Error processing {lock}: {e}")

    # Sort by ID (Folder Path) to ensure deterministic output order across builds
    library_lake.sort(key=lambda x: x["id"])

    print(f"Writing {len(library_lake)} albums to artifacts...")
    
    def write_artifact(path: Path):
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            # Indented JSON for readability as requested
            json.dump(library_lake, f, ensure_ascii=False, indent=2)
            f.write("\n")

    write_artifact(ui_dest)
    write_artifact(server_dest)
    
    elapsed = time.time() - start_time
    print(f"Build Complete in {elapsed:.2f}s.")
