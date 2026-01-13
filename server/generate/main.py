import os
import tomllib
from pathlib import Path
from tqdm import tqdm

from .extractor import PhysicalExtractor
from .engine import segregate_tags, render_toml_block, get_layout_keys

# Helper Registry
from .helpers import __all__ as PROTECTED_HELPERS
from .helpers import (
    track_path, cover_path, cover_byte_size, encoding, 
    bits_per_sample, channels, sample_rate, 
    duration_in_samples, duration_in_ms
)

def run_generate():
    config_path = Path("config.toml")
    if not config_path.exists():
        print("Error: config.toml not found.")
        return

    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    lib_root = Path(config["storage"]["library_root"]).expanduser().resolve()
    gen_cfg = config["generate"]
    supported_exts = gen_cfg["supported_extensions"]
    album_layout = gen_cfg["album"]["layout"]
    tracks_layout = gen_cfg["tracks"]["layout"]

    # Calculate "Opt-In" keys from config
    opted_in_keys = get_layout_keys(album_layout) | get_layout_keys(tracks_layout)

    folders_with_audio = set()
    for ext in supported_exts:
        for p in lib_root.rglob(f"*{ext}"):
            folders_with_audio.add(p.parent)

    for album_root in tqdm(folders_with_audio, desc="Compiling Library", unit="album"):
        
        # --- PHASE 1: THE BODY (files.toml) ---
        rel_paths = track_path.resolve(album_root, supported_exts)
        physics_tracks = []
        
        for rp in rel_paths:
            audio_obj, _ = PhysicalExtractor.get_audio_payload(album_root / rp)
            if audio_obj:
                physics_tracks.append({
                    "track_path": rp,
                    "encoding": encoding.resolve(audio_obj),
                    "bits_per_sample": bits_per_sample.resolve(audio_obj),
                    "channels": channels.resolve(audio_obj),
                    "sample_rate": sample_rate.resolve(audio_obj),
                    "duration_in_samples": duration_in_samples.resolve(audio_obj),
                    "duration_in_ms": duration_in_ms.resolve(audio_obj),
                })

        if not physics_tracks: continue

        cp = cover_path.resolve(album_root)
        c_size = cover_byte_size.resolve(album_root, cp)
        
        p_album_pool, p_track_pools = segregate_tags(physics_tracks, greedy=True)
        if cp:
            p_album_pool["cover_path"] = cp
            p_album_pool["cover_byte_size"] = c_size

        # Always overwrite files.toml
        with open(album_root / "files.toml", "w", encoding="utf-8") as f:
            f.write("[album]\n")
            f.write("\n".join(render_toml_block(p_album_pool)) + "\n\n")
            for tp in p_track_pools:
                f.write("[[tracks]]\n")
                f.write("\n".join(render_toml_block(tp)) + "\n\n")

        # --- PHASE 2: THE SOUL (metadata.toml) ---
        tag_pool_list = []
        for i, rp in enumerate(rel_paths):
            _, tags = PhysicalExtractor.get_audio_payload(album_root / rp)
            
            # INJECTION: Add calculated helpers to the pool so they CAN be opted-in
            track_physics = physics_tracks[i]
            for h_name in PROTECTED_HELPERS:
                if h_name in track_physics:
                    tags[h_name] = track_physics[h_name]

            # FILTERING: Remove any protected helper UNLESS it is opted-in via layout
            final_tags = {}
            for k, v in tags.items():
                if k in PROTECTED_HELPERS:
                    if k in opted_in_keys:
                        final_tags[k] = v
                else:
                    final_tags[k] = v
            
            tag_pool_list.append(final_tags)

        m_album_pool, m_track_pools = segregate_tags(tag_pool_list, layout=tracks_layout, greedy=False)

        # FORCED OVERWRITE: Removed existence check to ensure metadata.toml is always refreshed
        meta_path = album_root / "metadata.toml"
        with open(meta_path, "w", encoding="utf-8") as f:
            f.write("[album]\n")
            f.write("\n".join(render_toml_block(m_album_pool, album_layout)) + "\n\n")
            for tp in m_track_pools:
                f.write("[[tracks]]\n")
                f.write("\n".join(render_toml_block(tp, tracks_layout)) + "\n\n")

    print("\nCompilation Complete.")

if __name__ == "__main__":
    run_generate()
