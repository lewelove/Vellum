import ast

def parse_int(val) -> int:
    if val is None: return 0
    s = str(val).strip()
    if "/" in s: s = s.split("/")[0]
    return int(s) if s.isdigit() else 0

def is_match(key: str, harvest_val: any, lock_val: any, album_lock: dict = None) -> bool:
    s_harvest = str(harvest_val).strip() if harvest_val is not None else ""
    
    # 1. Check if the value is a real list (e.g. from JSON)
    if isinstance(lock_val, list):
        s_lock = "; ".join(str(v).strip() for v in lock_val)
    
    # 2. Check if the value is a stringified list "['a', 'b']" (common in your extensions)
    elif isinstance(lock_val, str) and lock_val.startswith("[") and lock_val.endswith("]"):
        try:
            # Safely evaluate the string into a Python list
            parsed_list = ast.literal_eval(lock_val)
            if isinstance(parsed_list, list):
                s_lock = "; ".join(str(v).strip() for v in parsed_list)
            else:
                s_lock = lock_val.strip()
        except (ValueError, SyntaxError):
            s_lock = lock_val.strip()
    else:
        s_lock = str(lock_val).strip() if lock_val is not None else ""

    # GLOBAL RULE: If tag is in lock but missing in physical file, ignore
    if s_lock and not s_harvest:
        return True

    # Rule: If it's a single disc release, ignore DISCNUMBER
    if key == "DISCNUMBER" and album_lock:
        if parse_int(album_lock.get("TOTALDISCS")) == 1:
            return True

    # Rule: Numeric comparison for Track/Disc
    if key in ("TRACKNUMBER", "DISCNUMBER"):
        return parse_int(s_harvest) == parse_int(s_lock)

    return s_harvest == s_lock
