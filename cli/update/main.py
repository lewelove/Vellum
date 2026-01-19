import tomllib
import sys
import hashlib
from pathlib import Path
from tqdm import tqdm

from .sentry import verify_trust, TrustState
from .compiler import compile_album
from .database import Database

def get_file_content_hash(path: Path) -> str:
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(8192):
            sha256.update(chunk)
    return sha256.hexdigest()

def run_update():
    config_path = Path("config.toml")
    if not config_path.exists():
        return

    # Simple flag parsing
    force_mode = "--force" in sys.argv

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    gen_cfg = config.get("generate", {})
    supported_exts = gen_cfg.get("supported_extensions", [".flac"])
    
    # Initialize Database
    # Using default path ~/.mpf2k/library.db
    db_path = Path("~/.mpf2k/library.db").expanduser().resolve()
    db = Database(db_path, config_path)

    anchors = list(lib_root.rglob("metadata.toml"))
    
    updates_count = 0
    processed_ids = []
    
    for anchor in tqdm(anchors, desc="Updating Library", unit="album"):
        album_root = anchor.parent
        
        # 1. Verify / Compile
        trust = verify_trust(album_root, force=force_mode)
        
        if trust != TrustState.VALID:
            compile_album(album_root, supported_exts, library_root=lib_root)
            updates_count += 1
            
        # 2. Database Sync
        lock_path = album_root / "metadata.lock"
        if lock_path.exists():
            try:
                # Calculate ID
                album_id = str(album_root.relative_to(lib_root))
                
                # Get Lock Hash for caching
                lock_hash = get_file_content_hash(lock_path)
                
                # Load Data
                with open(lock_path, "rb") as f:
                    lock_data = tomllib.load(f)
                
                # Sync
                db.sync_album(album_id, lock_data, lock_hash)
                
                # Mark as processed
                processed_ids.append(album_id)
                
            except Exception as e:
                tqdm.write(f"Error syncing DB for {album_root}: {e}")

    # 3. Prune Database (Garbage Collection)
    if processed_ids:
        db.prune(processed_ids)
    
    db.close()

    print(f"\nUpdate Complete. {updates_count} albums refreshed.")
