import sys
import json
import importlib.util
from pathlib import Path

def load_extensions(functions_dir):
    registry = {"album": {}, "tracks": {}}
    if not functions_dir.exists(): return registry
    for py_file in functions_dir.glob("*.py"):
        module_name = py_file.stem
        spec = importlib.util.spec_from_file_location(module_name, py_file)
        if not spec or not spec.loader: continue
        module = importlib.util.module_from_spec(spec)
        try:
            spec.loader.exec_module(module)
        except Exception as e:
            sys.stderr.write(f"Error loading extension {py_file.name}: {e}\n")
            continue
        for func_name in dir(module):
            if not func_name.startswith("resolve_"): continue
            func = getattr(module, func_name)
            if not callable(func): continue
            parts = func_name.split("_")
            if len(parts) < 4: continue
            scope, kind, key_raw = parts[1], parts[2], "_".join(parts[3:])
            if scope == "album": registry["album"][key_raw.lower()] = func
            elif scope == "track": registry["tracks"][key_raw.lower()] = func
    return registry

def main():
    original_stdout = sys.stdout
    sys.stdout = sys.stderr
    ext_functions = None
    compiler_registry = None
    for line in sys.stdin:
        if not line.strip(): continue
        try:
            pkg = json.loads(line)
            album_data = pkg.get("album", {})
            tracks_data = pkg.get("tracks", [])
            base_ctx = pkg.get("ctx", {})
            active_flags = base_ctx.get("active_flags", ["default"])
            if ext_functions is None:
                config = base_ctx.get("config", {})
                compiler_registry = config.get("compiler_registry", {})
                ext_cfg = config.get("extensions", {})
                f_dir = Path(ext_cfg.get("folder", "")).expanduser() / ext_cfg.get("functions_folder", "functions")
                ext_functions = load_extensions(f_dir)
            rust_paths = base_ctx.get("paths", {})
            a_root = Path(rust_paths.get("album_root")) if rust_paths.get("album_root") else None
            l_root = Path(rust_paths.get("library_root")) if rust_paths.get("library_root") else None
            album_ctx = {
                "album": album_data,
                "tracks": tracks_data,
                "config": base_ctx.get("config", {}),
                "source": base_ctx.get("metadata", {}).get("album", {}),
                "album_root": a_root,
                "library_root": l_root,
            }
            for key in list(album_data.keys()):
                reg_meta = compiler_registry.get(key, {})
                if reg_meta.get("provider") == "extension":
                    resolver = ext_functions["album"].get(key)
                    if resolver: album_data[key] = resolver(album_ctx)
            m_tracks = base_ctx.get("metadata", {}).get("tracks", [])
            h_tracks = base_ctx.get("harvest", [])
            for idx, track in enumerate(tracks_data):
                t_ctx = {
                    "track": track,
                    "album": album_data,
                    "config": base_ctx.get("config", {}),
                    "album_root": a_root,
                    "library_root": l_root,
                    "source": {**base_ctx.get("metadata", {}).get("album", {}), **(m_tracks[idx] if idx < len(m_tracks) else {})},
                    "physics": h_tracks[idx].get("physics", {}) if idx < len(h_tracks) else {},
                }
                for key in list(track.keys()):
                    reg_meta = compiler_registry.get(key, {})
                    if reg_meta.get("provider") == "extension":
                        resolver = ext_functions["tracks"].get(key)
                        if resolver: track[key] = resolver(t_ctx)
            original_stdout.write(json.dumps({"album": album_data, "tracks": tracks_data, "ctx": base_ctx}) + "\n")
            original_stdout.flush()
        except Exception as e:
            sys.stderr.write(f"Kernel Error: {e}\n")

if __name__ == "__main__":
    main()
