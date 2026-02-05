import tomllib
import json
import sys
import subprocess
from pathlib import Path
from tqdm import tqdm

from .engine import render_toml_block
from cli.update.compiler import harvest_metadata
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
    from cli.update.main import run_update

    config_path = Path("config.toml")
    if not config_path.exists():
        print("Error: config.toml not found.")
        return

    force_mode = "--force" in sys.argv

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    gen_cfg = config.get("generate", {})
    compress_cfg = config.get("compress", {})
    
    supported_exts = gen_cfg.get("supported_extensions", [".flac"])
    grouping_keys = gen_cfg.get("grouping_keys", ["ALBUMARTIST", "ALBUM"])
    
    album_layout = compress_cfg.get("album", {}).get("layout", [])
    tracks_layout = compress_cfg.get("tracks", {}).get("layout", [])

    if not force_mode:
        print("Refreshing existing locks...")
        run_update()

    print(f"Scanning library at: {lib_root}")
    
    tracked_paths = set()
    if not force_mode:
        lock_files = list(lib_root.rglob("metadata.lock.json"))
        for lf in lock_files:
            try:
                with open(lf, "r", encoding="utf-8") as f:
                    lock_data = json.load(f)
                    album_dir = lf.parent
                    for t in lock_data.get("tracks", []):
                        rel_path = t.get("track_path")
                        if rel_path:
                            abs_path = (album_dir / rel_path).resolve()
                            tracked_paths.add(str(abs_path))
            except Exception:
                continue

    print("Harvesting tags via Rust...")
    harvested_inventory = harvest_metadata(lib_root)
    
    raw_inventory = []
    for path_str, data in tqdm(harvested_inventory.items(), desc="Processing Harvest", unit="file"):
        if not force_mode and path_str in tracked_paths:
            continue
            
        tags = data["tags"]
        tags["track_path_absolute"] = path_str
        raw_inventory.append(tags)

    if not raw_inventory:
        print("No new files to process.")
        return

    print("Grouping tracks...")
    album_buckets = group_tracks(raw_inventory, grouping_keys)
    print(f"Found {len(album_buckets)} album groups to generate.")

    for group_id, tracks in tqdm(album_buckets.items(), desc="Generating Metadata", unit="album"):
        anchor_path, is_valid = resolve_anchor(tracks, str(lib_root), supported_exts)
        
        if not is_valid:
            tqdm.write(f"Skipping group {group_id}: Ecological Exclusivity violation.")
            continue

        sorted_tracks = sort_album_tracks(tracks)

        clean_tracks = []
        for t in sorted_tracks:
            clean = t.copy()
            if "track_path_absolute" in clean:
                del clean["track_path_absolute"]
            clean_tracks.append(clean)

        album_pool, track_pools = compress(
            clean_tracks, 
            tracks_layout=tracks_layout
        )
        
        meta_path = anchor_path / "metadata.toml"
        
        with open(meta_path, "w", encoding="utf-8") as f:
            f.write("[album]\n")
            f.write("\n".join(render_toml_block(album_pool, album_layout)) + "\n\n")
            
            for tp in track_pools:
                f.write("[[tracks]]\n")
                f.write("\n".join(render_toml_block(tp, tracks_layout)) + "\n\n")

        if sorted_tracks:
            first_track_path = Path(sorted_tracks[0]["track_path_absolute"])
            extract_cover_from_track(first_track_path, anchor_path / "cover")

    print("\nFinalizing locks for new metadata...")
    run_update()
    print("\nGeneration Complete.")
