import tomllib
import hashlib
from pathlib import Path

from .zipper import scan_physical_spine, zip_tracks
from .resolver import tags, physics, assets
from .writer import write_lock

def compile_album(album_root: Path, supported_exts: list):
    meta_path = album_root / "metadata.toml"
    
    with open(meta_path, "rb") as f:
        raw_meta = tomllib.load(f)
        
    sha256 = hashlib.sha256()
    with open(meta_path, "rb") as f:
        while chunk := f.read(8192):
            sha256.update(chunk)
    meta_hash = sha256.hexdigest()

    physical_spine = scan_physical_spine(album_root, supported_exts)

    album_defaults = raw_meta.get("album", {})
    raw_tracks = raw_meta.get("tracks", [])
    
    inflated_tracks = []
    
    if not raw_tracks and physical_spine:
        for _ in physical_spine:
            inflated_tracks.append(album_defaults.copy())
    else:
        for t in raw_tracks:
            inflated_tracks.append({**album_defaults, **t})

    zipped_tracks = zip_tracks(inflated_tracks, physical_spine)

    final_tracks = []
    unique_disc_numbers = set()
    
    for item in zipped_tracks:
        track_meta = item["meta"]
        phys_path = item["file"]
        
        track_data = tags.resolve_track_tags(track_meta, zipped_tracks.index(item))
        phys_data = physics.resolve_track_physics(album_root, phys_path)
        
        final_track = {**track_data, **phys_data}
        final_tracks.append(final_track)
        
        unique_disc_numbers.add(final_track.get("DISCNUMBER", "1"))

    album_data = tags.resolve_album_tags(album_defaults, len(final_tracks), len(unique_disc_numbers))
    album_data["metadata_toml_hash"] = meta_hash
    
    cover_data = assets.resolve_cover(album_root)
    album_data.update(cover_data)

    write_lock(album_root, album_data, final_tracks)
