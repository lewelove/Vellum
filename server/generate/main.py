import os
import tomllib
from pathlib import Path
from .extractor import FlacExtractor
from .sanitizer import slugify_album_filename

def load_config():
    config_path = Path("config.toml")
    with open(config_path, "rb") as f:
        return tomllib.load(f)

def run_generate():
    config = load_config()
    lib_root = Path(config["storage"]["library_root"]).expanduser()
    out_dir = Path(config["storage"]["metadata_store"]).expanduser()
    gen_cfg = config["generate"]

    out_dir.mkdir(parents=True, exist_ok=True)

    album_map = {} 
    
    for root, _, files in os.walk(lib_root):
        for file in files:
            if any(file.lower().endswith(ext) for ext in gen_cfg["supported_extensions"]):
                full_path = os.path.join(root, file)
                try:
                    extracted = FlacExtractor.extract(full_path)
                    tags = extracted["tags"]
                    
                    artist = tags.get("ALBUMARTIST") or tags.get("ARTIST", "Unknown")
                    album = tags.get("ALBUM", "Unknown")
                    key = (artist, album)
                    
                    if key not in album_map:
                        album_map[key] = {"tracks": [], "image": extracted["image_data"]}
                    
                    album_map[key]["tracks"].append(tags)
                    if not album_map[key]["image"]:
                        album_map[key]["image"] = extracted["image_data"]
                        
                except Exception as e:
                    print(f"Error processing {file}: {e}")

    for (artist, album_title), data in album_map.items():
        tracks = data["tracks"]
        
        # Sort tracks by disc and track number
        tracks.sort(key=lambda x: (
            x.get("DISCNUMBER", "1"), 
            int("".join(filter(str.isdigit, x.get("TRACKNUMBER", "0").split('/')[0]))) or 0
        ))

        all_keys = set().union(*(t.keys() for t in tracks))
        album_tags = {}

        for key in all_keys:
            values = [t.get(key) for t in tracks]
            # Identical across all tracks + UPPERCASE = Move to [album]
            if key.isupper() and all(v is not None for v in values) and len(set(values)) == 1:
                album_tags[key] = values[0]

        base_name = slugify_album_filename(gen_cfg["naming_file_pattern"], artist, album_title)
        
        toml_path = out_dir / f"metadata_{base_name}.toml"
        with open(toml_path, "w", encoding="utf-8") as f:
            f.write("[album]\n")
            for k in sorted(album_tags.keys()):
                f.write(f'{k} = "{album_tags[k]}"\n')
            f.write("\n")
            
            for t in tracks:
                f.write("[[tracks]]\n")
                unique_keys = [k for k in t.keys() if k not in album_tags]
                for k in sorted(unique_keys):
                    f.write(f'{k} = "{t[k]}"\n')
                f.write("\n")

        if data["image"]:
            with open(out_dir / f"cover_{base_name}.jpg", "wb") as f:
                f.write(data["image"])

if __name__ == "__main__":
    run_generate()
