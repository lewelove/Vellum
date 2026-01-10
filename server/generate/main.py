import tomllib
from pathlib import Path
from tqdm import tqdm

# Modules
from .harvester import scan_library
from .grouper import group_tracks, sort_album_tracks
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
    out_dir = Path(config["storage"]["metadata_store"]).expanduser()
    out_dir.mkdir(parents=True, exist_ok=True)
    
    gen_cfg = config["generate"]
    supported_exts = gen_cfg["supported_extensions"]
    
    grouping_keys = gen_cfg.get("grouping_keys", ["ALBUMARTIST", "ALBUM"])
    naming_sep = gen_cfg.get("naming_separator", "_-_")
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

    # 3. Process Groups (Pass 2: Generation + Targeted Image Extraction)
    for key_tuple, raw_track_list in tqdm(grouped.items(), desc="Generating Albums", unit="album"):
        
        # A. Sort tracks for consistent output
        sorted_tracks = sort_album_tracks(raw_track_list)
        
        # B. Generate Slug
        slug = generate_filename(list(key_tuple), naming_sep, sanitize_char)
        if not slug:
            slug = "unknown_group"

        # C. Targeted Image Extraction (Lead Track only)
        # We only read binary data for the first track of the album
        cover_image = None
        image_ext = "jpg" # default
        
        if sorted_tracks:
            lead_track_path = Path(sorted_tracks[0]["track_path_absolute"])
            # Deep read for image
            image_data_payload = FlacExtractor.extract(lead_track_path, include_image=True)
            
            if image_data_payload.get("image_data"):
                cover_image = image_data_payload["image_data"]
                # Determine extension from mime type
                mime = image_data_payload.get("mime_type", "")
                if "png" in mime:
                    image_ext = "png"
                elif "gif" in mime:
                    image_ext = "gif"

        # D. Logic Engine (Sieve)
        album_pool, track_pools = segregate_tags(sorted_tracks, tracks_layout)
        
        if cover_image:
            album_pool["cover_path"] = f"cover_{slug}.{image_ext}"

        # E. Render & Write TOML
        toml_path = out_dir / f"metadata_{slug}.toml"
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

        # F. Write Cover and Clear Memory
        if cover_image:
            with open(out_dir / f"cover_{slug}.{image_ext}", "wb") as f:
                f.write(cover_image)
            
            # Explicitly nullify to free binary RAM immediately
            cover_image = None
            image_data_payload = None

    print("\nGeneration complete.")

if __name__ == "__main__":
    run_generate()
