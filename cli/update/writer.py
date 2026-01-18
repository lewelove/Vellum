import json
from pathlib import Path
from cli.generate.compressor import get_layout_keys

def format_val(value):
    """Ensures values are TOML-compliant."""
    return json.dumps(value, ensure_ascii=False)

def render_lock_block(pool: dict, layout: list) -> list:
    """
    Renders a dictionary to TOML lines based on layout list.
    Supports inline '*' injection for appendix keys.
    """
    lines = []
    
    # 1. Identify consumed keys
    explicit_keys = get_layout_keys(layout) if layout else set()
    appendix_keys = sorted([k for k in pool.keys() if k not in explicit_keys])
    
    appendix_consumed = False

    def emit_appendix():
        nonlocal appendix_consumed
        if appendix_consumed:
            return
        for k in appendix_keys:
            lines.append(f'{k} = {format_val(pool[k])}')
        appendix_consumed = True

    # 2. Render Explicit Layout
    if layout:
        for item in layout:
            if isinstance(item, str):
                if item == "\n":
                    lines.append("")
                elif item == "*":
                    # INJECTION POINT: Dump appendix here
                    emit_appendix()
                elif item in pool:
                    lines.append(f'{item} = {format_val(pool[item])}')
            
            elif isinstance(item, dict):
                for header, tags in item.items():
                    has_content = False
                    # Lookahead
                    for t in tags:
                        if t == "*" and not appendix_consumed and appendix_keys:
                            has_content = True
                        elif t != "\n" and t in pool:
                            has_content = True
                    
                    if has_content:
                        if header: lines.append(header)
                        for t in tags:
                            if t == "\n":
                                lines.append("")
                            elif t == "*":
                                emit_appendix()
                            elif t in pool:
                                lines.append(f'{t} = {format_val(pool[t])}')

    # 3. Safety Net: If '*' was missing or didn't run, dump remaining keys at the end
    if not appendix_consumed:
        emit_appendix()
            
    return lines

def write_lock(album_root: Path, album_data: dict, tracks_data: list, layout_cfg: dict):
    """
    Writes the compiled metadata.lock using the provided layout config.
    """
    final_output = []
    
    alb_layout = layout_cfg.get("album", [])
    trk_layout = layout_cfg.get("tracks", [])

    # 1. Render Album Section
    final_output.append("[album]")
    final_output.extend(render_lock_block(album_data, alb_layout))

    # 2. Render Tracks Sections
    for track in tracks_data:
        final_output.append("[[tracks]]")
        final_output.extend(render_lock_block(track, trk_layout))

    # 3. Write to file
    lock_path = album_root / "metadata.lock"
    
    # Cleanup spacers
    cleaned_content = "\n".join(final_output)
    # Reduce triple+ newlines to double
    while "\n\n\n" in cleaned_content:
        cleaned_content = cleaned_content.replace("\n\n\n", "\n\n")
    
    with open(lock_path, "w", encoding="utf-8") as f:
        f.write(cleaned_content.strip() + "\n")
