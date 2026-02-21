import tomllib
import json
import sys
import os
import subprocess
from pathlib import Path
from tqdm import tqdm

from .engine import render_toml_block
from .compressor import compress
from .grouper import group_tracks, resolve_anchor, sort_album_tracks

def harvest_metadata(target_paths):
    """Call the native rust binary directly for harvesting."""
    if not isinstance(target_paths, list):
        target_paths = [target_paths]
    
    cmd = ["vellum", "harvest"] + [str(p) for p in target_paths]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    harvested_map = {}
    if result.returncode == 0:
        for line in result.stdout.strip().split("\n"):
            if not line: continue
            item = json.loads(line)
            harvested_map[str(Path(item["path"]).resolve())] = item
    return harvested_map

def run_generate():
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Error: config.toml not found.")
        return

    force_mode = "--force" in sys.argv

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    gen_cfg = config.get("generate", {})
    registry_cfg = config.get("compiler_registry", {})
    
    supported_exts = set(e.lower() for e in gen_cfg.get("supported_extensions", [".flac"]))
    grouping_keys = gen_cfg.get("grouping_keys", ["ALBUMARTIST", "ALBUM"])
    
    album_layout = registry_cfg.get("album", [])
    tracks_layout = registry_cfg.get("tracks", [])

    dirs_to_harvest = []
    for root, dirs, files in os.walk(lib_root):
        if not force_mode and "metadata.toml" in files:
            dirs[:] = []
            continue
            
        has_audio = False
        for f in files:
            if Path(f).suffix.lower() in supported_exts:
                has_audio = True
                break
        if has_audio:
            dirs_to_harvest.append(Path(root))

    if not dirs_to_harvest:
        print("No new audio directories found.")
        return

    harvested_inventory = harvest_metadata(dirs_to_harvest)
    if not harvested_inventory:
        return

    raw_inventory = []
    for path_str, data in harvested_inventory.items():
        tags = data["tags"]
        tags["track_path_absolute"] = path_str
        raw_inventory.append(tags)

    album_buckets = group_tracks(raw_inventory, grouping_keys)

    for group_id, tracks in tqdm(album_buckets.items(), desc="Generating Metadata", unit="album"):
        anchor_path, is_valid = resolve_anchor(tracks, str(lib_root), list(supported_exts))
        if not is_valid: continue

        sorted_tracks = sort_album_tracks(tracks)
        clean_tracks = []
        for t in sorted_tracks:
            clean = t.copy()
            if "track_path_absolute" in clean: del clean["track_path_absolute"]
            clean_tracks.append(clean)

        album_pool, track_pools = compress(clean_tracks, tracks_layout=tracks_layout)
        meta_path = anchor_path / "metadata.toml"
        
        with open(meta_path, "w", encoding="utf-8") as f:
            f.write("[album]\n")
            f.write("\n".join(render_toml_block(album_pool, album_layout)) + "\n\n")
            for tp in track_pools:
                f.write("[[tracks]]\n")
                f.write("\n".join(render_toml_block(tp, tracks_layout)) + "\n\n")

    # Trigger the NEW Rust-based update via subprocess
    print("\nGeneration complete. Triggering library update...")
    subprocess.run(["vellum", "update"])
