def resolve_album_tag_unix_added_primary(ctx):
    return str(ctx["source"].get("UNIX_ADDED_PRIMARY", ""))

def resolve_album_tag_unix_added_local(ctx):
    candidates = ["UNIX_ADDED_LOCAL", "UNIX_ADDED_PRIMARY"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return ""

def resolve_album_tag_unix_added_foobar(ctx):
    candidates = ["UNIX_ADDED_FOOBAR", "UNIXTIMEFOOBAR"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return ""

def resolve_album_tag_unix_added_applemusic(ctx):
    candidates = ["UNIX_ADDED_APPLEMUSIC", "UNIXTIMEAPPLE"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return ""

def resolve_album_tag_unix_added_youtube(ctx):
    candidates = ["UNIX_ADDED_YOUTUBE", "UNIXTIMEYOUTUBE"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return ""
