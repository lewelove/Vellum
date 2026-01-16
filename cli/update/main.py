import tomllib
import sys
from pathlib import Path
from tqdm import tqdm

from .sentry import verify_trust, TrustState
from .compiler import compile_album

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
    
    anchors = list(lib_root.rglob("metadata.toml"))
    
    updates_count = 0
    
    for anchor in tqdm(anchors, desc="Updating Library", unit="album"):
        album_root = anchor.parent
        
        # Pass the force flag to sentry
        trust = verify_trust(album_root, force=force_mode)
        
        if trust != TrustState.VALID:
            # Pass library_root to compiler for relative path calculations
            compile_album(album_root, supported_exts, library_root=lib_root)
            updates_count += 1

    print(f"\nUpdate Complete. {updates_count} albums refreshed.")
