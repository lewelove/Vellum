import tomllib
import sys
import json
import os
import time
import httpx
from pathlib import Path
from tqdm import tqdm

from .sentry import verify_trust, TrustState
from .compiler import compile_album

CACHE_FILE = Path("~/.mpf2k/cache.json").expanduser().resolve()

def load_cache():
    if CACHE_FILE.exists():
        try:
            with open(CACHE_FILE, "r") as f:
                return json.load(f)
        except:
            pass
    return {}

def save_cache(cache):
    CACHE_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(CACHE_FILE, "w") as f:
        json.dump(cache, f)

def notify_server(album_root: Path):
    """
    Sends a fire-and-forget notification to the running server.
    """
    url = "http://127.0.0.1:8000/api/internal/reload"
    try:
        httpx.post(url, params={"path": str(album_root)}, timeout=0.5)
    except httpx.RequestError:
        # Server might be down, which is fine for offline updates
        pass

def run_update():
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Config not found")
        return

    force_mode = "--force" in sys.argv
    
    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    gen_cfg = config.get("generate", {})
    supported_exts = gen_cfg.get("supported_extensions", [".flac"])
    
    sentry_cache = load_cache()
    new_cache = {}
    
    anchors = list(lib_root.rglob("metadata.toml"))
    
    updates_count = 0
    skips_count = 0
    
    print(f"Scanning {len(anchors)} albums...")

    for anchor in tqdm(anchors, desc="Verifying Library", unit="album"):
        album_root = anchor.parent
        album_path_str = str(album_root)
        
        # --- SENTRY FAST CHECK (Folder Mtime) ---
        try:
            current_mtime = int(os.path.getmtime(album_root))
        except OSError:
            current_mtime = 0
            
        cached_info = sentry_cache.get(album_path_str, {})
        cached_mtime = cached_info.get("mtime", 0)
        
        should_process = force_mode
        if not should_process:
            if current_mtime == 0 or current_mtime != cached_mtime:
                should_process = True

        if not should_process:
            # Cache hit
            new_cache[album_path_str] = cached_info
            skips_count += 1
            continue

        # --- DEEP CHECK (Compiler Logic) ---
        trust = verify_trust(album_root, force=force_mode)
        
        if trust != TrustState.VALID:
            compile_album(album_root, supported_exts, library_root=lib_root)
            
            # --- HOT RELOAD TRIGGER ---
            # Notify server immediately after compilation
            notify_server(album_root)
            
            updates_count += 1
            
        new_cache[album_path_str] = {"mtime": current_mtime}

    save_cache(new_cache)

    print(f"\nUpdate Complete. {updates_count} albums refreshed. {skips_count} albums skipped (cached).")
