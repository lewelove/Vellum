import json
from pathlib import Path

def format_val(value):
    return json.dumps(value, ensure_ascii=False)

def write_lock(album_root: Path, album_data: dict, tracks_data: list):
    lines = []
    
    lines.append("[album]")
    for k in sorted(album_data.keys()):
        lines.append(f"{k} = {format_val(album_data[k])}")
    lines.append("")

    for t in tracks_data:
        lines.append("[[tracks]]")
        for k in sorted(t.keys()):
            lines.append(f"{k} = {format_val(t[k])}")
        lines.append("")

    lock_path = album_root / "metadata.lock"
    with open(lock_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
