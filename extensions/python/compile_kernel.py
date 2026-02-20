import sys
import json
import os
import importlib.util
from pathlib import Path

def load_extensions(functions_dir):
    """
    Locates and indexes only the custom functions in the extensions directory.
    No legacy vellum code is imported or registered.
    """
    registry = {
        "ALBUM_TAGS": {},
        "ALBUM_HELPERS": {},
        "TRACK_TAGS": {},
        "TRACK_HELPERS": {}
    }

    if not functions_dir.exists():
        return registry

    for py_file in functions_dir.glob("*.py"):
        module_name = py_file.stem
        spec = importlib.util.spec_from_file_location(module_name, py_file)
        if not spec or not spec.loader:
            continue
        
        module = importlib.util.module_from_spec(spec)
        try:
            spec.loader.exec_module(module)
        except Exception as e:
            sys.stderr.write(f"Error loading extension {py_file.name}: {e}\n")
            continue

        for func_name in dir(module):
            if not func_name.startswith("resolve_"):
                continue
            
            func = getattr(module, func_name)
            if not callable(func):
                continue

            parts = func_name.split("_")
            if len(parts) < 4:
                continue

            scope = parts[1] # "album" or "track"
            kind = parts[2]  # "tag" or "helper"
            key_raw = "_".join(parts[3:])

            if scope == "album" and kind == "tag":
                registry["ALBUM_TAGS"][key_raw.upper()] = func
            elif scope == "album" and kind == "helper":
                registry["ALBUM_HELPERS"][key_raw.lower()] = func
            elif scope == "track" and kind == "tag":
                registry["TRACK_TAGS"][key_raw.upper()] = func
            elif scope == "track" and kind == "helper":
                registry["TRACK_HELPERS"][key_raw.lower()] = func

    return registry

def main():
    # Capture original stdout for JSON communication
    original_stdout = sys.stdout
    sys.stdout = sys.stderr

    registry = None

    for line in sys.stdin:
        if not line.strip():
            continue
        
        try:
            manifest = json.loads(line)
            album_data = manifest.get("album", {})
            tracks_data = manifest.get("tracks", [])
            base_ctx = manifest.get("ctx", {})
            
            if registry is None:
                ext_cfg = base_ctx.get("config", {}).get("extensions", {})
                functions_dir = Path(ext_cfg.get("folder", "")).expanduser() / ext_cfg.get("functions_folder", "functions")
                registry = load_extensions(functions_dir)

            rust_paths = base_ctx.get("paths", {})
            album_root = Path(rust_paths.get("album_root")) if rust_paths.get("album_root") else None
            library_root = Path(rust_paths.get("library_root")) if rust_paths.get("library_root") else None

            # Context construction for Album scope
            album_ctx = {
                "album": album_data,
                "tracks": tracks_data,
                "config": base_ctx.get("config", {}),
                "source": base_ctx.get("metadata", {}).get("album", {}),
                "album_root": album_root,
                "library_root": library_root,
            }
            
            # Enforce custom overrides only. 
            # If a key isn't in our extension registry, the Rust value is preserved.
            for key in list(album_data.keys()):
                resolver = registry["ALBUM_TAGS"].get(key) or registry["ALBUM_HELPERS"].get(key.lower())
                if resolver:
                    album_data[key] = resolver(album_ctx)

            m_tracks = base_ctx.get("metadata", {}).get("tracks", [])
            h_tracks = base_ctx.get("harvest", [])

            # Context construction for Track scope
            for idx, track in enumerate(tracks_data):
                harvest_item = h_tracks[idx] if idx < len(h_tracks) else {}
                
                t_ctx = {
                    "track": track,
                    "album": album_data,
                    "config": base_ctx.get("config", {}),
                    "album_root": album_root,
                    "library_root": library_root,
                    "source": {
                        **base_ctx.get("metadata", {}).get("album", {}), 
                        **(m_tracks[idx] if idx < len(m_tracks) else {}),
                    },
                    "physics": harvest_item.get("physics", {}),
                    "raw_tags": harvest_item.get("tags", {}),
                    "track_path_resolved": (album_root / harvest_item.get("track_path")).resolve() if harvest_item.get("track_path") else None
                }
                
                for key in list(track.keys()):
                    resolver = registry["TRACK_TAGS"].get(key) or registry["TRACK_HELPERS"].get(key.lower())
                    if resolver:
                        track[key] = resolver(t_ctx)

            # Emit the final enriched object
            original_stdout.write(json.dumps({
                "album": album_data,
                "tracks": tracks_data,
                "ctx": base_ctx
            }) + "\n")
            original_stdout.flush()

        except Exception as e:
            sys.stderr.write(f"Kernel Error: {e}\n")

if __name__ == "__main__":
    main()
