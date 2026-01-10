import tomllib
from pathlib import Path

# Modules
from .harvester import scan_library
from .grouper import group_tracks, sort_album_tracks
from .naming import generate_filename
from .engine import segregate_tags, process_layout

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
    sanitize_char = gen_cfg.get("naming_sanitization_char", "_")
    naming_pattern = gen_cfg["naming_file_pattern"]
    
    album_layout = gen_cfg["album"]["layout"]
    tracks_layout = gen_cfg["tracks"]["layout"]

    # 1. Harvest
    all_tracks = scan_library(lib_root, supported_exts)
    if not all_tracks:
        print("No tracks found.")
        return

    # 2. Group
    grouped = group_tracks(all_tracks)
    print(f"grouped into {len(grouped)} albums.")

    # 3. Process Groups
    for key_tuple, raw_track_list in grouped.items():
        artist_key, album_key, custom_id = key_tuple
        
        # A. Sort
        sorted_tracks = sort_album_tracks(raw_track_list)
        
        # B. Find Cover (First non-null image in the sorted pool)
        cover_image = None
        for t in sorted_tracks:
            if t.get("_embedded_image"):
                cover_image = t["_embedded_image"]
                break
        
        # C. Generate Slug
        slug = generate_filename(
            naming_pattern, 
            artist_key, 
            album_key, 
            custom_id, 
            sanitize_char
        )
        
        # D. Logic Engine (Sieve)
        # Note: segregate_tags strips _embedded_image internally
        album_pool, track_pools = segregate_tags(sorted_tracks, tracks_layout)
        
        # Inject metadata about the cover if it exists
        if cover_image:
            album_pool["cover_path"] = f"cover_{slug}.jpg"

        # E. Render & Write
        toml_path = out_dir / f"metadata_{slug}.toml"
        
        with open(toml_path, "w", encoding="utf-8") as f:
            # [album]
            f.write("[album]\n")
            album_lines = process_layout(album_pool, album_layout)
            f.write("\n".join(album_lines))
            f.write("\n\n")
            
            # [[tracks]]
            for t_pool in track_pools:
                f.write("[[tracks]]\n")
                track_lines = process_layout(t_pool, tracks_layout)
                f.write("\n".join(track_lines))
                f.write("\n\n")

        # F. Write Cover
        if cover_image:
            with open(out_dir / f"cover_{slug}.jpg", "wb") as f:
                f.write(cover_image)
        
        print(f"Generated: {slug}")

if __name__ == "__main__":
    run_generate()
