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

# --- ALBUM SCOPE ---

# ALBUMARTIST
def resolve_tag_albumartist(ctx):
    return str(ctx["source"].get("ALBUMARTIST", "Unknown"))

# ALBUM
def resolve_tag_album(ctx):
    return str(ctx["source"].get("ALBUM", "Unknown"))

# DATE
def resolve_tag_date(ctx):
    return str(ctx["source"].get("DATE", "0000"))

# GENRE
def resolve_tag_genre(ctx):
    return str(ctx["source"].get("GENRE", "Unknown"))

# TOTALTRACKS
def resolve_tag_totaltracks(ctx):
    # Context must provide the full inflated list or calculated count
    return ctx.get("total_tracks_count", 0)

# TOTALDISCS
def resolve_tag_totaldiscs(ctx):
    return ctx.get("total_discs_count", 0)

# ORIGINAL_YYYY_MM
def resolve_tag_original_yyyy_mm(ctx):
    val = ctx["source"].get("ORIGINAL_YYYY_MM")
    if val: return str(val)
    date_val = resolve_tag_date(ctx)
    return f"{date_val[:4]}-00"

# ORIGINAL_YEAR
def resolve_tag_original_year(ctx):
    # Depends on ORIGINAL_YYYY_MM
    # In a pure key-loop, we might not have the resolved value yet if order matters.
    # We re-derive or use source. The SSOT says: calculate from ORIGINAL_YYYY_MM.
    # For safety in this stateless loop, we re-call the resolver or check source.
    yyyy_mm = resolve_tag_original_yyyy_mm(ctx)
    return yyyy_mm[:4]

# ORIGINAL_DATE
def resolve_tag_original_date(ctx):
    yyyy_mm = resolve_tag_original_yyyy_mm(ctx)
    return _format_human_date(yyyy_mm)

# RELEASE_YYYY_MM
def resolve_tag_release_yyyy_mm(ctx):
    val = ctx["source"].get("RELEASE_YYYY_MM")
    if val: return str(val)
    date_val = resolve_tag_date(ctx)
    return f"{date_val[:4]}-00"

# RELEASE_YEAR
def resolve_tag_release_year(ctx):
    yyyy_mm = resolve_tag_release_yyyy_mm(ctx)
    return yyyy_mm[:4]

# RELEASE_DATE
def resolve_tag_release_date(ctx):
    yyyy_mm = resolve_tag_release_yyyy_mm(ctx)
    return _format_human_date(yyyy_mm)

# COUNTRY
def resolve_tag_country(ctx):
    return str(ctx["source"].get("COUNTRY", ""))

# LABEL
def resolve_tag_label(ctx):
    return str(ctx["source"].get("LABEL", ""))

# CATALOGNUMBER
def resolve_tag_catalognumber(ctx):
    return str(ctx["source"].get("CATALOGNUMBER", ""))

# MEDIA
def resolve_tag_media(ctx):
    return str(ctx["source"].get("MEDIA", ""))

# COMMENT
def resolve_tag_comment(ctx):
    val = ctx["source"].get("COMMENT")
    if val: return str(val)
    
    # Fallback logic: RELEASE_YEAR + COUNTRY + LABEL + CATALOGNUMBER
    parts = [
        resolve_tag_release_year(ctx),
        resolve_tag_country(ctx),
        resolve_tag_label(ctx),
        resolve_tag_catalognumber(ctx)
    ]
    return " ".join([p for p in parts if p]).strip()

# UNIX_ADDED_PRIMARY
def resolve_tag_unix_added_primary(ctx):
    return str(ctx["source"].get("UNIX_ADDED_PRIMARY", ""))

# UNIX_ADDED_LOCAL
def resolve_tag_unix_added_local(ctx):
    return str(ctx["source"].get("UNIX_ADDED_LOCAL", ""))

# UNIX_ADDED_APPLEMUSIC
def resolve_tag_unix_added_applemusic(ctx):
    return str(ctx["source"].get("UNIX_ADDED_APPLEMUSIC", ""))

# UNIX_ADDED_YOUTUBE
def resolve_tag_unix_added_youtube(ctx):
    return str(ctx["source"].get("UNIX_ADDED_YOUTUBE", ""))

# CUSTOM_ID
def resolve_tag_custom_id(ctx):
    return str(ctx["source"].get("CUSTOM_ID", ""))

# CUSTOM_ALBUMARTIST
def resolve_tag_custom_albumartist(ctx):
    val = ctx["source"].get("CUSTOM_ALBUMARTIST")
    if val: return str(val)
    return resolve_tag_albumartist(ctx)

# CUSTOM_STRING
def resolve_tag_custom_string(ctx):
    return str(ctx["source"].get("CUSTOM_STRING", ""))

# OLD_COMMENT
def resolve_tag_old_comment(ctx):
    return str(ctx["source"].get("OLD_COMMENT", ""))

# DISCOGS_URL
def resolve_tag_discogs_url(ctx):
    return str(ctx["source"].get("DISCOGS_URL", ""))

# MUSICBRAINZ_URL
def resolve_tag_musicbrainz_url(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_URL", ""))

# CTDBID
def resolve_tag_ctdbid(ctx):
    return str(ctx["source"].get("CTDBID", ""))

# ACCURIPID
def resolve_tag_accuripid(ctx):
    return str(ctx["source"].get("ACCURIPID", ""))

# DISCID
def resolve_tag_discid(ctx):
    return str(ctx["source"].get("DISCID", ""))

# MUSICBRAINZ_ALBUMARTISTID
def resolve_tag_musicbrainz_albumartistid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ALBUMARTISTID", ""))

# MUSICBRAINZ_RELEASEGROUPID
def resolve_tag_musicbrainz_releasegroupid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_RELEASEGROUPID", ""))

# MUSICBRAINZ_ALBUMID
def resolve_tag_musicbrainz_albumid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ALBUMID", ""))

# --- TRACK SCOPE ---

# TITLE
def resolve_tag_title(ctx):
    return str(ctx["source"].get("TITLE", "Untitled"))

# ARTIST
def resolve_tag_artist(ctx):
    val = ctx["source"].get("ARTIST")
    if val: return str(val)
    return str(ctx["source"].get("ALBUMARTIST", "Unknown"))

# TRACKNUMBER
def resolve_tag_tracknumber(ctx):
    # This acts as a getter for the compiled value or fallback to source
    # In Phase 3, we already established a hard 'TRACKNUMBER' in the dict.
    return str(ctx["source"].get("TRACKNUMBER", ""))

# DISCNUMBER
def resolve_tag_discnumber(ctx):
    return str(ctx["source"].get("DISCNUMBER", "1"))

# MUSICBRAINZ_ARTISTID
def resolve_tag_musicbrainz_artistid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ARTISTID", ""))

# MUSICBRAINZ_RELEASETRACKID
def resolve_tag_musicbrainz_releasetrackid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_RELEASETRACKID", ""))

# MUSICBRAINZ_TRACKID
def resolve_tag_musicbrainz_trackid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_TRACKID", ""))

