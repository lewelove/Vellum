import tomllib
from pathlib import Path
from tqdm import tqdm

from .synchronizer import sync_album

def run_update():
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Error: config.toml not found.")
        return

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    # 1. Setup
    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    gen_cfg = config["generate"]
    supported_exts = gen_cfg["supported_extensions"]

    # 2. Anchor Discovery (Fast Scan)
    # We look for metadata.toml because that defines an "Album" in our system.
    anchors = list(lib_root.rglob("metadata.toml"))

    print(f"Found {len(anchors)} albums anchored in: {lib_root}")

    # 3. The Update Loop
    updates_count = 0
    for anchor in tqdm(anchors, desc="Updating Library", unit="album"):
        album_root = anchor.parent
        
        # Pass control to the Synchronizer
        was_updated = sync_album(album_root, supported_exts)
        
        if was_updated:
            updates_count += 1

    print(f"\nUpdate Complete. {updates_count} albums refreshed.")
