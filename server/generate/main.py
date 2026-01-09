import os
import tomllib
from pathlib import Path
from .extractor import FlacExtractor
from .sanitizer import slugify_album_filename
from .engine import segregate_tags, process_layout

def load_config():
    config_path = Path("config.toml")
    with open(config_path, "rb") as f:
        return tomllib.load(f)

def run_generate():
    config = load_config()
    lib_root = Path(config["storage"]["library_root"]).expanduser()
    out_dir = Path(config["storage"]["metadata_store"]).expanduser()
    
    gen_cfg = config["generate"]
    supported_exts = gen_cfg["supported_extensions"]
    
    album_layout = gen_cfg["album"]["layout"]
    tracks_layout = gen_cfg["tracks"]["layout"]
    
    out_dir.mkdir(parents=True, exist_ok=True)

    # Walk and Group
    for root, _, files in os.walk(lib_root):
        audio_files = [f for f in files if any(f.lower().endswith(ext) for ext in supported_exts)]
        
        if not audio_files:
            continue

        # Process one folder = one album
        full_root = Path(root)
        raw_tracks = []
        cover_image = None
        
        # 1. Extract
        for f in audio_files:
            file_path = full_root / f
            data = FlacExtractor.extract(file_path)
            if data and data.get("tags"):
                raw_tracks.append(data["tags"])
                # Grab first available image
                if not cover_image and data.get("image_data"):
                    cover_image = data["image_data"]

        if not raw_tracks:
            continue

        # Sort tracks by filename to ensure deterministic processing order
        raw_tracks.sort(key=lambda x: x["track_path"])

        # 2. Logic Engine (Sieve)
        album_pool, track_pools = segregate_tags(raw_tracks, tracks_layout)

        # 3. Determine Filename
        artist = album_pool.get("ALBUMARTIST") or album_pool.get("ARTIST") or "Unknown"
        album_title = album_pool.get("ALBUM") or "Unknown"
        base_name = slugify_album_filename(
            gen_cfg["naming_file_pattern"], 
            str(artist), 
            str(album_title),
            gen_cfg.get("naming_sanitization_char", "_")
        )
        
        # Inject Cover Path into Album Pool
        if cover_image:
            album_pool["cover_path"] = f"cover_{base_name}.jpg"

        # 4. Render & Write
        toml_path = out_dir / f"metadata_{base_name}.toml"
        
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

        # 5. Write Cover
        if cover_image:
            with open(out_dir / f"cover_{base_name}.jpg", "wb") as f:
                f.write(cover_image)
        
        print(f"Generated: {base_name}")

if __name__ == "__main__":
    run_generate()
