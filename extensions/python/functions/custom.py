def resolve_album_tag_custom_id(ctx):
    return str(ctx["source"].get("CUSTOM_ID", ""))

def resolve_album_tag_custom_albumartist(ctx):
    candidates = ["CUSTOM_ALBUMARTIST", "ARTISTARTIST", "ALBUMARTIST"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return "Unknown"

def resolve_album_tag_custom_string(ctx):
    candidates = ["CUSTOM_STRING", "CUSTOMSTRING"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return ""

def resolve_album_tag_old_comment(ctx):
    return str(ctx["source"].get("OLD_COMMENT", ""))
