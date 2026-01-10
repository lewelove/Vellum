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
    
    # New Logic Extraction
    grouping_keys = gen_cfg.get("grouping_keys", ["ALBUMARTIST", "ALBUM"])
    naming_sep = gen_cfg.get("naming_separator", "_-_")
    sanitize_char = gen_cfg.get("naming_sanitization_char", "_")
    
    album_layout = gen_cfg["album"]["layout"]
    tracks_layout = gen_cfg["tracks"]["layout"]

    # 1. Harvest
    all_tracks = scan_library(lib_root, supported_exts)
    if not all_tracks:
        print("No tracks found.")
        return

    # 2. Group
    # We now pass the dynamic keys from config
    grouped = group_tracks(all_tracks, grouping_keys)
    print(f"grouped into {len(grouped)} albums.")

    # 3. Process Groups
    # key_tuple is now an ordered tuple corresponding to grouping_keys
    for key_tuple, raw_track_list in grouped.items():
        
        # A. Sort
        sorted_tracks = sort_album_tracks(raw_track_list)
        
        # B. Find Cover (First non-null image in the sorted pool)
        cover_image = None
        for t in sorted_tracks:
            if t.get("_embedded_image"):
                cover_image = t["_embedded_image"]
                break
        
        # C. Generate Slug
        # The key_tuple contains the values we grouped by. 
        # We pass these directly to the naming engine.
        slug = generate_filename(
            list(key_tuple), 
            naming_sep,
            sanitize_char
        )
        
        # Fallback for completely empty keys (e.g. tracks with no tags)
        if not slug:
            slug = "unknown_group"

        # D. Logic Engine (Sieve)
        album_pool, track_pools = segregate_tags(sorted_tracks, tracks_layout)
        
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
