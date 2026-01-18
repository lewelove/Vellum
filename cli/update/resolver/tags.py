import datetime

# --- UTILS ---

def _format_human_date(yyyy_mm):
    if not yyyy_mm or yyyy_mm == "0000-00":
        return "Unknown Date"
    parts = yyyy_mm.split("-")
    if len(parts) < 2 or parts[1] == "00":
        return parts[0]
    try:
        dt = datetime.datetime.strptime(yyyy_mm, "%Y-%m")
        return dt.strftime("%B %Y")
    except ValueError:
        return parts[0]

# --- ALBUM TAGS ---

def resolve_album_tag_albumartist(ctx):
    return str(ctx["source"].get("ALBUMARTIST", "Unknown"))

def resolve_album_tag_album(ctx):
    return str(ctx["source"].get("ALBUM", "Unknown"))

def resolve_album_tag_genre(ctx):
    return str(ctx["source"].get("GENRE", "Unknown"))

def resolve_album_tag_date(ctx):
# Compatability compliant tag (used to display original release year in other players)
# Acts like a universal fallback due to common use
    candidates = ["DATE", "YEAR", "ORIGINALYEAR"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return "0000"

def resolve_album_tag_original_yyyy_mm(ctx):
    candidates = ["ORIGINAL_YYYY_MM", "ORIGINALYEARMONTH"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    date_val = resolve_album_tag_date(ctx)
    return f"{date_val[:4]}-00"

def resolve_album_tag_original_year(ctx):
    yyyy_mm = resolve_album_tag_original_yyyy_mm(ctx)
    return yyyy_mm[:4]

def resolve_album_tag_original_date(ctx):
    yyyy_mm = resolve_album_tag_original_yyyy_mm(ctx)
    return _format_human_date(yyyy_mm)

def resolve_album_tag_release_yyyy_mm(ctx):
    val = ctx["source"].get("RELEASE_YYYY_MM")
    if val: return str(val)
    date_val = resolve_album_tag_date(ctx)
    return f"{date_val[:4]}-00"

def resolve_album_tag_release_year(ctx):
    yyyy_mm = resolve_album_tag_release_yyyy_mm(ctx)
    return yyyy_mm[:4]

def resolve_album_tag_release_date(ctx):
    yyyy_mm = resolve_album_tag_release_yyyy_mm(ctx)
    return _format_human_date(yyyy_mm)

def resolve_album_tag_totaltracks(ctx):
    return str(ctx.get("total_tracks_count", 0))

def resolve_album_tag_totaldiscs(ctx):
    return str(ctx.get("total_discs_count", 0))

# --- TRACK TAGS ---

def resolve_track_tag_title(ctx):
    return str(ctx["source"].get("TITLE", "Untitled"))

def resolve_track_tag_artist(ctx):
    val = ctx["source"].get("ARTIST")
    if val: return str(val)
    return str(ctx["source"].get("ALBUMARTIST", "Unknown"))

def resolve_track_tag_tracknumber(ctx):
    return str(ctx["source"].get("TRACKNUMBER", ""))

def resolve_track_tag_discnumber(ctx):
    return str(ctx["source"].get("DISCNUMBER", "1"))

def resolve_track_tag_lyrics(ctx):
    return str(ctx["source"].get("LYRICS", ""))

