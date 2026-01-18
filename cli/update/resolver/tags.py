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

def resolve_album_tag_date(ctx):
    candidates = ["ORIGINAL_RELEASE_DATE", "DATE", "YEAR", "RELEASE_DATE"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return "0000"

def resolve_album_tag_genre(ctx):
    return str(ctx["source"].get("GENRE", "Unknown"))

def resolve_album_tag_totaltracks(ctx):
    return str(ctx.get("total_tracks_count", 0))

def resolve_album_tag_totaldiscs(ctx):
    return str(ctx.get("total_discs_count", 0))

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

def resolve_album_tag_unix_added_primary(ctx):
    return str(ctx["source"].get("UNIX_ADDED_PRIMARY", ""))

def resolve_album_tag_unix_added_local(ctx):
    candidates = ["UNIX_ADDED_LOCAL", "UNIXTIMEFOOBAR"]
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

def resolve_album_tag_custom_id(ctx):
    return str(ctx["source"].get("CUSTOM_ID", ""))

def resolve_album_tag_custom_albumartist(ctx):
    val = ctx["source"].get("CUSTOM_ALBUMARTIST")
    if val: return str(val)
    return resolve_album_tag_albumartist(ctx)

def resolve_album_tag_custom_string(ctx):
    candidates = ["CUSTOM_STRING", "CUSTOMSTRING"]
    for key in candidates:
        val = ctx["source"].get(key)
        if val: 
            return str(val)
    return ""

def resolve_album_tag_old_comment(ctx):
    return str(ctx["source"].get("OLD_COMMENT", ""))

def resolve_album_tag_discogs_url(ctx):
    return str(ctx["source"].get("DISCOGS_URL", ""))

def resolve_album_tag_musicbrainz_url(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_URL", ""))

def resolve_album_tag_ctdbid(ctx):
    return str(ctx["source"].get("CTDBID", ""))

def resolve_album_tag_accuripid(ctx):
    return str(ctx["source"].get("ACCURIPID", ""))

def resolve_album_tag_discid(ctx):
    return str(ctx["source"].get("DISCID", ""))

def resolve_album_tag_musicbrainz_albumartistid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ALBUMARTISTID", ""))

def resolve_album_tag_musicbrainz_releasegroupid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_RELEASEGROUPID", ""))

def resolve_album_tag_musicbrainz_albumid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ALBUMID", ""))

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

def resolve_track_tag_musicbrainz_artistid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ARTISTID", ""))

def resolve_track_tag_musicbrainz_releasetrackid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_RELEASETRACKID", ""))

def resolve_track_tag_musicbrainz_trackid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_TRACKID", ""))
