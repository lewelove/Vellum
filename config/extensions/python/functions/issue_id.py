def resolve_album_tag_country(ctx):
    return str(ctx["source"].get("COUNTRY", ""))

def resolve_album_tag_label(ctx):
    return str(ctx["source"].get("LABEL", ""))

def resolve_album_tag_catalognumber(ctx):
    return str(ctx["source"].get("CATALOGNUMBER", ""))

def resolve_album_tag_media(ctx):
    return str(ctx["source"].get("MEDIA", ""))

def resolve_album_tag_comment(ctx):
    # First priority: literal comment in metadata.toml
    val = ctx["source"].get("COMMENT")
    if val: 
        return str(val)
    
    country = resolve_album_tag_country(ctx)
    label = resolve_album_tag_label(ctx)
    cat_no = resolve_album_tag_catalognumber(ctx)

    if not any([country, label, cat_no]):
        return ""

    # Utilize Pass 1 resolution from Rust for the release date
    yyyy_mm = str(ctx.get("album", {}).get("RELEASE_YYYY_MM", "0000-00"))
    parts = [
        yyyy_mm[:4],
        country,
        label,
        cat_no
    ]
    return " ".join([p for p in parts if p]).strip()

def resolve_track_tag_ctdbid(ctx):
    return str(ctx["source"].get("CTDBID", ""))

def resolve_track_tag_accuripid(ctx):
    return str(ctx["source"].get("ACCURIPID", ""))

def resolve_track_tag_discid(ctx):
    return str(ctx["source"].get("DISCID", ""))
