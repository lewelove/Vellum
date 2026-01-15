import datetime

def _format_human_date(yyyy_mm: str) -> str:
    if yyyy_mm == "0000-00":
        return "Unknown Date"
    parts = yyyy_mm.split("-")
    if len(parts) < 2 or parts[1] == "00":
        return parts[0]
    try:
        dt = datetime.datetime.strptime(yyyy_mm, "%Y-%m")
        return dt.strftime("%B %Y")
    except ValueError:
        return parts[0]

# ALBUMARTIST
def resolve_tag_albumartist(source: dict) -> str:
    return str(source.get("ALBUMARTIST", "Unknown"))

# ALBUM
def resolve_tag_album(source: dict) -> str:
    return str(source.get("ALBUM", "Untitled"))

# ARTIST
def resolve_tag_artist(source: dict) -> str:
    return str(source.get("ARTIST", albumartist))

# TITLE
def resolve_tag_title(source: dict) -> str:
    return str(source.get("TITLE", "Untitled"))

# DATE
def resolve_tag_date(source: dict) -> str:
    return str(source.get("DATE", "0000"))

# GENRE
def resolve_tag_genre(source: dict) -> str:
    return str(source.get("GENRE", "Unknown"))

# TOTALTRACKS
def resolve_tag_totaltracks(tracks: list) -> int:
    return len(tracks)

# TOTALDISCS
def resolve_tag_totaldiscs(tracks: list) -> int:
    discs = set()
    for t in tracks:
        discs.add(str(t.get("DISCNUMBER", "1")))
    return len(discs)

# ORIGINAL_YYYY_MM
def resolve_tag_original_yyyy_mm(source: dict) -> str:
    if "ORIGINAL_YYYY_MM" in source:
        return str(source["ORIGINAL_YYYY_MM"])
    return f"{resolve_tag_date(source)[:4]}-00"

# ORIGINAL_YEAR
def resolve_tag_original_year(source: dict) -> str:
    return resolve_tag_original_yyyy_mm(source)[:4]

# ORIGINAL_DATE
def resolve_tag_original_date(source: dict) -> str:
    return _format_human_date(resolve_tag_original_yyyy_mm(source))

# RELEASE_YYYY_MM
def resolve_tag_release_yyyy_mm(source: dict) -> str:
    if "RELEASE_YYYY_MM" in source:
        return str(source["RELEASE_YYYY_MM"])
    return f"{resolve_tag_date(source)[:4]}-00"

# RELEASE_YEAR
def resolve_tag_release_year(source: dict) -> str:
    return resolve_tag_release_yyyy_mm(source)[:4]

# RELEASE_DATE
def resolve_tag_release_date(source: dict) -> str:
    return _format_human_date(resolve_tag_release_yyyy_mm(source))

# COUNTRY
def resolve_tag_country(source: dict) -> str:
    return str(source.get("COUNTRY", ""))

# LABEL
def resolve_tag_label(source: dict) -> str:
    return str(source.get("LABEL", ""))

# CATALOGNUMBER
def resolve_tag_catalognumber(source: dict) -> str:
    return str(source.get("CATALOGNUMBER", ""))

# MEDIA
def resolve_tag_media(source: dict) -> str:
    return str(source.get("MEDIA", ""))

# COMMENT
def resolve_tag_comment(source: dict, release_year: str, country: str, label: str, catalog: str) -> str:
    val = source.get("COMMENT")
    if val:
        return str(val)
    parts = [release_year, country, label, catalog]
    return " ".join([p for p in parts if p]).strip()

# UNIX_ADDED_PRIMARY
def resolve_tag_unix_added_primary(source: dict) -> str:
    return str(source.get("UNIX_ADDED_PRIMARY", ""))

# UNIX_ADDED_LOCAL
def resolve_tag_unix_added_local(source: dict) -> str:
    return str(source.get("UNIX_ADDED_LOCAL", ""))

# UNIX_ADDED_APPLEMUSIC
def resolve_tag_unix_added_applemusic(source: dict) -> str:
    return str(source.get("UNIX_ADDED_APPLEMUSIC", ""))

# UNIX_ADDED_YOUTUBE
def resolve_tag_unix_added_youtube(source: dict) -> str:
    return str(source.get("UNIX_ADDED_YOUTUBE", ""))

# CUSTOM_ID
def resolve_tag_custom_id(source: dict) -> str:
    return str(source.get("CUSTOM_ID", ""))

# CUSTOM_ALBUMARTIST
def resolve_tag_custom_albumartist(source: dict, albumartist: str) -> str:
    return str(source.get("CUSTOM_ALBUMARTIST", albumartist))

# CUSTOM_STRING
def resolve_tag_custom_string(source: dict) -> str:
    return str(source.get("CUSTOM_STRING", ""))

# OLD_COMMENT
def resolve_tag_old_comment(source: dict) -> str:
    return str(source.get("OLD_COMMENT", ""))

# DISCOGS_URL
def resolve_tag_discogs_url(source: dict) -> str:
    return str(source.get("DISCOGS_URL", ""))

# MUSICBRAINZ_URL
def resolve_tag_musicbrainz_url(source: dict) -> str:
    return str(source.get("MUSICBRAINZ_URL", ""))

# CTDBID
def resolve_tag_ctdbid(source: dict) -> str:
    return str(source.get("CTDBID", ""))

# ACCURIPID
def resolve_tag_accuripid(source: dict) -> str:
    return str(source.get("ACCURIPID", ""))

# DISCID
def resolve_tag_discid(source: dict) -> str:
    return str(source.get("DISCID", ""))

# MUSICBRAINZ_ALBUMARTISTID
def resolve_tag_musicbrainz_albumartistid(source: dict) -> str:
    return str(source.get("MUSICBRAINZ_ALBUMARTISTID", ""))

# MUSICBRAINZ_RELEASEGROUPID
def resolve_tag_musicbrainz_releasegroupid(source: dict) -> str:
    return str(source.get("MUSICBRAINZ_RELEASEGROUPID", ""))

# MUSICBRAINZ_ALBUMID
def resolve_tag_musicbrainz_albumid(source: dict) -> str:
    return str(source.get("MUSICBRAINZ_ALBUMID", ""))

# MUSICBRAINZ_ARTISTID
def resolve_tag_musicbrainz_albumartistid(source: dict) -> str:
    return str(source.get("MUSICBRAINZ_ARTISTID", ""))

# MUSICBRAINZ_RELEASETRACKID
def resolve_tag_musicbrainz_releasegroupid(source: dict) -> str:
    return str(source.get("MUSICBRAINZ_RELEASETRACKID", ""))

# MUSICBRAINZ_TRACKID
def resolve_tag_musicbrainz_albumid(source: dict) -> str:
    return str(source.get("MUSICBRAINZ_TRACKID", ""))
