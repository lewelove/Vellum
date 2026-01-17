import json
from cli.generate.compressor import get_layout_keys

def format_toml_value(value):
    return json.dumps(value, ensure_ascii=False)

def render_toml_block(pool: dict, layout: list = None) -> list:
    """
    Renders a dictionary to TOML lines based on a layout list.
    Entries in 'pool' that are not in 'layout' are treated as 'appendix'.
    The appendix is inserted at the position of '*' or at the end if '*' is missing.
    """
    lines = []
    
    # 1. Partition keys
    explicit_keys = get_layout_keys(layout) if layout else set()
    appendix_keys = sorted([k for k in pool.keys() if k not in explicit_keys])
    
    appendix_consumed = False

    def emit_appendix():
        nonlocal appendix_consumed
        if appendix_consumed:
            return
        for k in appendix_keys:
            lines.append(f'{k} = {format_toml_value(pool[k])}')
        appendix_consumed = True

    if not layout:
        emit_appendix()
        return lines

    for item in layout:
        if isinstance(item, str):
            if item == "\n":
                lines.append("")
            elif item == "*":
                emit_appendix()
            elif item.startswith("#"):
                lines.append(item)
            elif item in pool:
                lines.append(f'{item} = {format_toml_value(pool[item])}')
        
        elif isinstance(item, dict):
            for header, tags in item.items():
                has_content = False
                for t in tags:
                    if t == "*" and not appendix_consumed and appendix_keys:
                        has_content = True
                    elif t in pool:
                        has_content = True

                if has_content:
                    if header: lines.append(header)
                    for t in tags:
                        if t == "\n":
                            lines.append("")
                        elif t == "*":
                            emit_appendix()
                        elif t in pool:
                            lines.append(f'{t} = {format_toml_value(pool[t])}')

    if not appendix_consumed:
        emit_appendix()
            
    return lines
