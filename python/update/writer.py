import json
from pathlib import Path
from python.generate.compressor import get_layout_keys

def format_val(value):
    """Ensures values are TOML-compliant strings for the TOML writer."""
    return json.dumps(value, ensure_ascii=False)

def render_toml_block(pool: dict, layout: list) -> list:
    """
    Renders a dictionary to TOML lines based on layout list.
    Preserves the human-readable .lock format.
    """
    lines = []
    
    explicit_keys = get_layout_keys(layout) if layout else set()
    appendix_keys = sorted([k for k in pool.keys() if k not in explicit_keys])
    
    appendix_consumed = False

    def emit_appendix():
        nonlocal appendix_consumed
        if appendix_consumed: return
        for k in appendix_keys:
            lines.append(f'{k} = {format_val(pool[k])}')
        appendix_consumed = True

    if layout:
        for item in layout:
            if isinstance(item, str):
                if item == "\n": lines.append("")
                elif item == "*": emit_appendix()
                elif item in pool: lines.append(f'{item} = {format_val(pool[item])}')
            elif isinstance(item, dict):
                for header, tags in item.items():
                    has_content = False
                    for t in tags:
                        if t == "*" and not appendix_consumed and appendix_keys: has_content = True
                        elif t != "\n" and t in pool: has_content = True
                    
                    if has_content:
                        if header: lines.append(header)
                        for t in tags:
                            if t == "\n": lines.append("")
                            elif t == "*": emit_appendix()
                            elif t in pool: lines.append(f'{t} = {format_val(pool[t])}')

    if not appendix_consumed:
        emit_appendix()
            
    return lines

def write_lock(album_root: Path, album_data: dict, tracks_data: list, layout_cfg: dict):
    """
    Writes TWO lock files:
    1. metadata.lock.toml (Human Readable, Legacy Compatible)
    2. metadata.lock.json (Machine Optimized, Compiler Artifact)
    """
    
    toml_lines = []
    alb_layout = layout_cfg.get("album", [])
    trk_layout = layout_cfg.get("tracks", [])

    toml_lines.append("[album]")
    toml_lines.extend(render_toml_block(album_data, alb_layout))

    for track in tracks_data:
        toml_lines.append("[[tracks]]")
        toml_lines.extend(render_toml_block(track, trk_layout))

    cleaned_content = "\n".join(toml_lines)
    while "\n\n\n" in cleaned_content:
        cleaned_content = cleaned_content.replace("\n\n\n", "\n\n")
    
    lock_toml_path = album_root / "metadata.lock.toml"
    with open(lock_toml_path, "w", encoding="utf-8") as f:
        f.write(cleaned_content.strip() + "\n")

    lock_object = {
        "album": album_data,
        "tracks": tracks_data
    }

    lock_json_path = album_root / "metadata.lock.json"
    with open(lock_json_path, "w", encoding="utf-8") as f:
        json.dump(lock_object, f, ensure_ascii=False, indent=2, sort_keys=True)
        f.write("\n")
