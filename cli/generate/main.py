import tomllib
from pathlib import Path
from tqdm import tqdm

from .extractor import PhysicalExtractor
from .engine import segregate_tags, render_toml_block
from .grouper import group_tracks, resolve_anchor, sort_album_tracks

def extract_cover_from_track(track_path: Path, destination_base: Path):
    """
    Lazy extraction: Opens the file strictly to extract the embedded picture.
    """
    try:
        audio, _ = PhysicalExtractor.get_audio_payload(track_path)
        
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
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Error: config.toml not found.")
        return

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    gen_cfg = config["generate"]
    
    supported_exts = gen_cfg.get("supported_extensions", [".flac"])
    grouping_keys = gen_cfg.get("grouping_keys", ["ALBUMARTIST", "ALBUM"])
    
    album_layout = gen_cfg.get("album", {}).get("layout", [])
    tracks_layout = gen_cfg.get("tracks", {}).get("layout", [])

    print(f"Scanning library at: {lib_root}")
    
    raw_inventory = []
    files_found = []
    
    for ext in supported_exts:
        files_found.extend(lib_root.rglob(f"*{ext}"))
    
    for file_path in tqdm(files_found, desc="Harvesting Tags", unit="file"):
        audio_obj, tags = PhysicalExtractor.get_audio_payload(file_path)
        
        if tags:
            tags["track_path_absolute"] = str(file_path)
            raw_inventory.append(tags)
        
        del audio_obj

    if not raw_inventory:
        print("No supported audio files found or no tags detected.")
        return

    print("Grouping tracks...")
    album_buckets = group_tracks(raw_inventory, grouping_keys)
    print(f"Found {len(album_buckets)} unique album groups.")

    for group_id, tracks in tqdm(album_buckets.items(), desc="Generating Metadata", unit="album"):
        
        anchor_path, is_valid = resolve_anchor(tracks, lib_root)
        
        if not is_valid:
            tqdm.write(f"Skipping group {group_id}: Invalid anchor or depth > 2.")
            continue

        sorted_tracks = sort_album_tracks(tracks)

        album_pool, track_pools = segregate_tags(
            sorted_tracks, 
            album_layout=album_layout, 
            tracks_layout=tracks_layout, 
            greedy=False
        )
        
        if "track_path_absolute" in album_pool: 
            del album_pool["track_path_absolute"]
            
        for t in track_pools:
            if "track_path_absolute" in t: 
                del t["track_path_absolute"]

        meta_path = anchor_path / "metadata.toml"
        
        with open(meta_path, "w", encoding="utf-8") as f:
            f.write("[album]\n")
            f.write("\n".join(render_toml_block(album_pool, album_layout)) + "\n\n")
            
            for tp in track_pools:
                f.write("[[tracks]]\n")
                f.write("\n".join(render_toml_block(tp, tracks_layout)) + "\n\n")

        cover_candidates = ["cover.jpg", "cover.png", "folder.jpg", "folder.png"]
        has_cover = any((anchor_path / c).exists() for c in cover_candidates)
        
        if not has_cover and sorted_tracks:
            first_track_path = Path(sorted_tracks[0]["track_path_absolute"])
            
            extract_cover_from_track(first_track_path, anchor_path / "cover")

    print("\nGeneration Complete.")
