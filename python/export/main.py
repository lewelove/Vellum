import tomllib
import shutil
from pathlib import Path
from tqdm import tqdm

from python.generate.naming import generate_filename
from python.generate.engine import render_toml_block

def run_export():
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Error: config.toml not found.")
        return

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    export_root = Path(config["storage"]["library_export"]).expanduser().resolve()
    
    gen_cfg = config["generate"]
    group_keys = gen_cfg.get("grouping_keys", ["ALBUMARTIST", "ALBUM"])
    naming_sep = gen_cfg.get("naming_separator", "_")
    
    export_root.mkdir(parents=True, exist_ok=True)

    album_folders = [p.parent for p in lib_root.rglob("metadata.toml")]

    print(f"Exporting {len(album_folders)} albums to: {export_root}")

    for album_path in tqdm(album_folders, desc="Exporting", unit="album"):
        meta_path = album_path / "metadata.toml"
        files_path = album_path / "files.toml"

        tqdm.write(f"Processing: {album_path.relative_to(lib_root)}")

        with open(meta_path, "rb") as f:
            meta_data = tomllib.load(f)
            
        with open(files_path, "rb") as f:
            files_data = tomllib.load(f)

        meta_album = meta_data.get("album", {})
        slug_components = []
        for key in group_keys:
            val = meta_album.get(key, "")
            if val:
                slug_components.append(str(val))
        
        slug = generate_filename(slug_components, naming_sep)

        merged_album = {**meta_album, **files_data.get("album", {})}
        
        m_tracks = meta_data.get("tracks", [])
        f_tracks = files_data.get("tracks", [])
        merged_tracks = []
        
        for m_t, f_t in zip(m_tracks, f_tracks):
            merged_tracks.append({**m_t, **f_t})

        export_toml_path = export_root / f"metadata+files_{slug}.toml"
        with open(export_toml_path, "w", encoding="utf-8") as f:
            f.write("[album]\n")
            f.write("\n".join(render_toml_block(merged_album)) + "\n\n")
            for t in merged_tracks:
                f.write("[[tracks]]\n")
                f.write("\n".join(render_toml_block(t)) + "\n\n")

        cover_rel = files_data.get("album", {}).get("cover_path")
        if cover_rel:
            src_cover = album_path / cover_rel
            if src_cover.exists():
                ext = src_cover.suffix
                dest_cover = export_root / f"cover_{slug}{ext}"
                shutil.copy2(src_cover, dest_cover)

    print("\nExport complete.")
