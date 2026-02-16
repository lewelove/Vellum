import tomllib
import sys
import json
import os
import hashlib
import httpx
from pathlib import Path
from tqdm import tqdm

from .sentry import verify_trust, TrustState
from .compiler import compile_album
from .harvester import harvest_metadata

def get_base_cache_dir() -> Path:
    return Path("~/.vellum/libraries_cache").expanduser().resolve()

def get_cache_path(library_root: Path) -> Path:
    base_dir = get_base_cache_dir()
    root_hash = hashlib.sha256(str(library_root).encode()).hexdigest()
    return base_dir / f"{root_hash}.json"

def load_cache(cache_path: Path):
    if cache_path.exists():
        try:
            with open(cache_path, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            pass
    return {}

def save_cache(cache: dict, cache_path: Path):
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    with open(cache_path, "w", encoding="utf-8") as f:
        json.dump(cache, f, indent=2)

def notify_server(album_root: Path):
    url = "http://127.0.0.1:8000/api/internal/reload"
    try:
        httpx.post(url, params={"path": str(album_root)}, timeout=0.5)
    except httpx.RequestError:
        pass

def trigger_server_reset():
    url = "http://127.0.0.1:8000/api/internal/reset"
    try:
        httpx.post(url, timeout=2.0)
    except httpx.RequestError:
        pass

def run_update():
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Config not found")
        return

    force_mode = "--force" in sys.argv
    
    with open(config_path, "rb") as f:
        config_data = tomllib.load(f)

    lib_root = Path(config_data["storage"]["library_root"]).expanduser().resolve()
    current_lib_hash = hashlib.sha256(str(lib_root).encode()).hexdigest()
    
    base_cache = get_base_cache_dir()
    current_json_path = base_cache / "current.json"
    
    needs_reset = False
    if current_json_path.exists():
        try:
            with open(current_json_path, "r", encoding="utf-8") as f:
                saved = json.load(f)
                if saved.get("hash") != current_lib_hash:
                    needs_reset = True
        except Exception:
            needs_reset = True
    else:
        needs_reset = True

    if needs_reset:
        base_cache.mkdir(parents=True, exist_ok=True)
        with open(current_json_path, "w", encoding="utf-8") as f:
            json.dump({"hash": current_lib_hash}, f)
        trigger_server_reset()

    gen_cfg = config_data.get("generate", {})
    supported_exts = gen_cfg.get("supported_extensions", [".flac"])
    
    cache_path = get_cache_path(lib_root)
    sentry_cache = load_cache(cache_path)
    new_cache = {}
    
    anchors = list(lib_root.rglob("metadata.toml"))
    work_queue = []
    
    print(f"Scanning {len(anchors)} albums...")

    skips_count = 0
    for anchor in tqdm(anchors, desc="Verifying Library", unit="album"):
        album_root = anchor.parent
        album_path_str = str(album_root)
        
        try:
            folder_mtime = int(os.path.getmtime(album_root))
            meta_mtime = int(os.path.getmtime(anchor))
            current_mtime_sum = folder_mtime + meta_mtime
        except OSError:
            current_mtime_sum = 0
            
        cached_info = sentry_cache.get(album_path_str, {})
        cached_mtime = cached_info.get("mtime_sum", 0)
        
        should_process = force_mode
        if not should_process:
            if current_mtime_sum == 0 or current_mtime_sum != cached_mtime:
                should_process = True

        if not should_process:
            new_cache[album_path_str] = cached_info
            skips_count += 1
            continue

        trust = verify_trust(album_root, force=force_mode)
        
        if trust != TrustState.VALID:
            work_queue.append((album_root, current_mtime_sum))
        else:
            new_cache[album_path_str] = {"mtime_sum": current_mtime_sum}
            skips_count += 1

    harvest_map = None
    if force_mode and work_queue:
        print("Bulk Harvesting Metadata (Force Mode)...")
        try:
            harvest_map = harvest_metadata(lib_root)
            print(f"Harvested {len(harvest_map)} tracks.")
        except Exception as e:
            print(f"Bulk Harvest Failed: {e}. Falling back to atomic harvesting.")
            harvest_map = None

    updates_count = 0
    if work_queue:
        for album_root, mtime_sum in tqdm(work_queue, desc="Compiling Updates", unit="album"):
            try:
                compile_album(
                    album_root, 
                    supported_exts, 
                    library_root=lib_root, 
                    pre_harvested_data=harvest_map
                )
                
                notify_server(album_root)
                new_cache[str(album_root)] = {"mtime_sum": mtime_sum}
                updates_count += 1
            except Exception as e:
                print(f"Error compiling {album_root}: {e}")

    save_cache(new_cache, cache_path)

    print(f"\nUpdate Complete. {updates_count} albums refreshed. {skips_count} albums skipped (cached).")
