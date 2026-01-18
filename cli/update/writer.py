import json
from pathlib import Path
from cli.generate.compressor import get_layout_keys

def format_val(value):
    """Ensures values are TOML-compliant."""
    return json.dumps(value, ensure_ascii=False)

def render_lock_block(pool: dict, layout: list) -> list:
    """
    Renders a dictionary to TOML lines based on layout list.
    Handles *, \n, headers, and simple strings.
    Always appends unconsumed keys at the end.
    """
    lines = []
    
    # 1. Identify consumed keys
    explicit_keys = get_layout_keys(layout) if layout else set()
    
    # 2. Render Explicit Layout
    if layout:
        for item in layout:
            if isinstance(item, str):
                if item == "\n":
                    lines.append("")
                elif item == "*":
                    # Placeholder for split, handled by appendix check
                    continue 
                elif item in pool:
                    lines.append(f'{item} = {format_val(pool[item])}')
            
            elif isinstance(item, dict):
                for header, tags in item.items():
                    has_content = False
                    # Lookahead to see if we need the header
                    for t in tags:
                        if t != "*" and t != "\n" and t in pool:
                            has_content = True
                    
                    if has_content:
                        if header: lines.append(header)
                        for t in tags:
                            if t == "\n":
                                lines.append("")
                            elif t in pool:
                                lines.append(f'{t} = {format_val(pool[t])}')

    # 3. Render Appendix (Unconsumed Keys)
    # This logic covers both the "*" behavior and the safety fallback
    appendix_keys = sorted([k for k in pool.keys() if k not in explicit_keys])
    
    if appendix_keys:
        # Add spacer if we had content before
        if lines and lines[-1] != "":
            lines.append("")
            
        for k in appendix_keys:
            lines.append(f'{k} = {format_val(pool[k])}')
            
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
