import os
import tomllib
from pathlib import Path
from tqdm import tqdm

# Modules
from .harvester import scan_library
from .grouper import group_tracks, sort_album_tracks, resolve_anchor
from .naming import generate_filename
from .engine import segregate_tags, process_layout
from .extractor import FlacExtractor

def load_config():
    config_path = Path("config.toml")
    with open(config_path, "rb") as f:
        return tomllib.load(f)

def run_generate():
    config = load_config()
    
    # Config Extraction
    lib_root = config["storage"]["library_root"]
    fallback_store = Path(config["storage"]["metadata_store"]).expanduser()
    fallback_store.mkdir(parents=True, exist_ok=True)
    
    gen_cfg = config["generate"]
    supported_exts = gen_cfg["supported_extensions"]
    
    grouping_keys = gen_cfg.get("grouping_keys", ["ALBUMARTIST", "ALBUM"])
    naming_sep = gen_cfg.get("naming_separator", "_")
    sanitize_char = gen_cfg.get("naming_sanitization_char", "_")
    
    album_layout = gen_cfg["album"]["layout"]
    tracks_layout = gen_cfg["tracks"]["layout"]

    # 1. Harvest (Pass 1: Text Only)
    all_tracks = scan_library(lib_root, supported_exts)
    if not all_tracks:
        print("No tracks found.")
        return

    # 2. Group
    grouped = group_tracks(all_tracks, grouping_keys)
    print(f"Grouped into {len(grouped)} potential albums.")

    # 3. Process Groups
    for key_tuple, raw_track_list in tqdm(grouped.items(), desc="Generating Albums", unit="album"):
        
        # A. Sort tracks
        sorted_tracks = sort_album_tracks(raw_track_list)
        
        # B. Calculate Anchor & Logic Mode
        anchor, is_valid_anchor = resolve_anchor(sorted_tracks, lib_root)
        
        # C. Determine Output Paths based on Mode
        if is_valid_anchor:
            # MODE A: In-Place (Preferred)
            out_dir = anchor
            toml_name = "metadata.toml"
            cover_name_base = "cover"
            use_relative_paths = True
        else:
            # MODE B: Fallback (Scattered)
            slug = generate_filename(list(key_tuple), naming_sep, sanitize_char)
            if not slug: slug = "unknown_group"
            
            out_dir = fallback_store
            toml_name = f"metadata_{slug}.toml"
            cover_name_base = f"cover_{slug}"
            use_relative_paths = False

        # D. Targeted Image Extraction (Lead Track only)
        cover_image = None
        image_ext = "jpg" # default
        
        if sorted_tracks:
            lead_track_path = Path(sorted_tracks[0]["track_path_absolute"])
            image_data_payload = FlacExtractor.extract(lead_track_path, include_image=True)
            
            if image_data_payload.get("image_data"):
                cover_image = image_data_payload["image_data"]
                mime = image_data_payload.get("mime_type", "")
                if "png" in mime: image_ext = "png"
                elif "gif" in mime: image_ext = "gif"

        # E. Prepare Pathing for TOML
        # We modify the track objects in memory to reflect the path style needed
        for t in sorted_tracks:
            abs_path = t["track_path_absolute"]
            if use_relative_paths:
                # Relative to the anchor (metadata.toml location)
                # e.g. "CD1/01.flac" or "01.flac"
                t["track_path"] = os.path.relpath(abs_path, out_dir)
            else:
                # Absolute path for fallback store
                t["track_path"] = abs_path

        # F. Logic Engine (Sieve)
        album_pool, track_pools = segregate_tags(sorted_tracks, tracks_layout)
        
        # Add cover path to album pool if exists
        if cover_image:
            album_pool["cover_path"] = f"{cover_name_base}.{image_ext}"

        # G. Write TOML
        toml_path = out_dir / toml_name
        try:
            with open(toml_path, "w", encoding="utf-8") as f:
                f.write("[album]\n")
                album_lines = process_layout(album_pool, album_layout)
                f.write("\n".join(album_lines))
                f.write("\n\n")
                
                for t_pool in track_pools:
                    f.write("[[tracks]]\n")
                    track_lines = process_layout(t_pool, tracks_layout)
                    f.write("\n".join(track_lines))
                    f.write("\n\n")
        except PermissionError:
            print(f"Skipping write: Permission denied for {toml_path}")
            continue

        # H. Write Cover
        if cover_image:
            cover_path = out_dir / f"{cover_name_base}.{image_ext}"
            try:
                with open(cover_path, "wb") as f:
                    f.write(cover_image)
            except PermissionError:
                pass # Fail silently on image write permission
            
            cover_image = None

    print("\nGeneration complete.")

if __name__ == "__main__":
    run_generate()
