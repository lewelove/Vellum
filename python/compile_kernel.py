import sys
import json
from pathlib import Path
from python.update.resolver import setup_registry, find_resolver, get_registered_keys
from python.update.zipper import parse_int

def get_layout_keys(layout):
    keys = set()
    if not layout:
        return keys
    for item in layout:
        if isinstance(item, str):
            if not item.startswith("#") and item not in ("\n", "*"):
                keys.add(item)
        elif isinstance(item, dict):
            for val in item.values():
                keys.update(get_layout_keys(val))
    return keys

def main():
    original_stdout = sys.stdout
    sys.stdout = sys.stderr

    try:
        raw_manifest = sys.stdin.read()
        if not raw_manifest:
            sys.exit(0)
        manifest = json.loads(raw_manifest)
    except Exception as e:
        sys.stderr.write(f"Kernel Error: Failed to parse manifest: {e}\n")
        sys.exit(1)

    config = manifest["config"]
    metadata = manifest["metadata"]
    harvest = manifest["harvest"]
    paths = manifest["paths"]

    project_root = Path(paths["project_root"])
    album_root = Path(paths["album_root"])
    library_root = Path(paths["library_root"])

    ext_folder = config.get("compiler", {}).get("extensions_folder")
    ext_config = config.get("compiler", {}).get("extensions", {})
    setup_registry(ext_folder, ext_config)

    A_TAGS, A_HELPERS, T_TAGS, T_HELPERS = get_registered_keys()
    
    album_layout = config.get("lock", {}).get("layout", {}).get("album", [])
    track_layout = config.get("lock", {}).get("layout", {}).get("tracks", [])

    album_source = metadata.get("album", {})
    track_sources = metadata.get("tracks", [])

    album_keys_in_layout = get_layout_keys(album_layout)
    track_keys_in_layout = get_layout_keys(track_layout)

    inflated_tracks = []
    if not track_sources:
        for _ in harvest:
            inflated_tracks.append(album_source.copy())
    else:
        for t in track_sources:
            inflated_tracks.append({**album_source, **t})

    inflated_tracks.sort(key=lambda t: (
        parse_int(t.get("DISCNUMBER", "1")), 
        parse_int(t.get("TRACKNUMBER", "0"))
    ))

    # Resolve Tracks
    final_tracks = []
    for idx, track_source in enumerate(inflated_tracks):
        h_file = harvest[idx]
        t_path_abs = Path(h_file["path"])
        
        if "track_path" not in track_source:
            track_source["track_path"] = h_file.get("track_path", "")

        ctx = {
            "source": track_source,
            "album_root": album_root,
            "library_root": library_root,
            "track_path_resolved": t_path_abs,
            "physics": h_file.get("physics", {}),
            "raw_tags": h_file.get("tags", {}),
            "config": config
        }

        final_track = {}
        all_possible_t_keys = set(track_keys_in_layout) | set(T_TAGS) | set(T_HELPERS)
        
        for key in all_possible_t_keys:
            resolver = find_resolver(key, "TRACK_TAGS") or find_resolver(key.lower(), "TRACK_HELPERS")
            if resolver:
                final_track[key] = resolver(ctx)
            elif key in track_keys_in_layout:
                final_track[key] = str(track_source.get(key, ""))
        
        final_tracks.append(final_track)

    unique_discs = set(t.get("DISCNUMBER", "1") for t in final_tracks)

    album_ctx = {
        "source": album_source,
        "album_root": album_root,
        "library_root": library_root,
        "metadata_toml_hash": paths.get("metadata_toml_hash", ""),
        "metadata_toml_mtime": paths.get("metadata_toml_mtime", 0),
        "cover_hash": paths.get("cover_hash", ""),
        "total_tracks_count": len(final_tracks),
        "total_discs_count": len(unique_discs),
        "all_tracks_final": final_tracks,
        "config": config
    }

    # Resolve Album
    final_album = {}
    all_possible_a_keys = set(album_keys_in_layout) | set(A_TAGS) | set(A_HELPERS)
    for key in all_possible_a_keys:
        resolver = find_resolver(key, "ALBUM_TAGS") or find_resolver(key.lower(), "ALBUM_HELPERS")
        if resolver:
            final_album[key] = resolver(album_ctx)
        elif key in album_keys_in_layout:
            final_album[key] = str(album_source.get(key, ""))

    sys.stdout = original_stdout
    sys.stdout.write(json.dumps({
        "album": final_album,
        "tracks": final_tracks
    }))

if __name__ == "__main__":
    main()
