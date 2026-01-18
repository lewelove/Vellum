def resolve_album_tag_country(ctx):
    return str(ctx["source"].get("COUNTRY", ""))

def resolve_album_tag_label(ctx):
    return str(ctx["source"].get("LABEL", ""))

def resolve_album_tag_catalognumber(ctx):
    return str(ctx["source"].get("CATALOGNUMBER", ""))

def resolve_album_tag_media(ctx):
    return str(ctx["source"].get("MEDIA", ""))

def resolve_album_tag_comment(ctx):
    val = ctx["source"].get("COMMENT")
    if val: return str(val)
    
    parts = [
        resolve_album_tag_release_year(ctx),
        resolve_album_tag_country(ctx),
        resolve_album_tag_label(ctx),
        resolve_album_tag_catalognumber(ctx)
    ]
    return " ".join([p for p in parts if p]).strip()

def resolve_album_tag_ctdbid(ctx):
    return str(ctx["source"].get("CTDBID", ""))

def resolve_album_tag_accuripid(ctx):
    return str(ctx["source"].get("ACCURIPID", ""))

def resolve_album_tag_discid(ctx):
    return str(ctx["source"].get("DISCID", ""))

