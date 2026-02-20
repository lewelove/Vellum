import tomllib
import json
import sys
import os
from pathlib import Path
from tqdm import tqdm

from .engine import render_toml_block
from python.update.harvester import harvest_metadata
from .compressor import compress
from .grouper import group_tracks, resolve_anchor, sort_album_tracks

def extract_cover_from_track(track_path: Path, destination_base: Path):
    try:
        from mutagen.flac import FLAC
        audio = FLAC(track_path)
        if not audio or not audio.pictures:
            return

        pic = audio.pictures[0]
        ext = ".png" if pic.mime == "image/png" else ".jpg"
        final_dest = destination_base.with_suffix(ext)
        
        with open(final_dest, "wb") as f:
            f.write(pic.data)
    except Exception:
        pass

def run_generate():
    from python.update.main import run_update

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
    
    # Use compiler_registry as the layout source for generate/compress
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

        if sorted_tracks:
            extract_cover_from_track(Path(sorted_tracks[0]["track_path_absolute"]), anchor_path / "cover")

    run_update()
