import datetime

def resolve_album_tags(source: dict, total_tracks: int, total_discs: int) -> dict:
    out = {}
    
    out["ALBUMARTIST"] = str(source.get("ALBUMARTIST", "Unknown"))
    out["ALBUM"] = str(source.get("ALBUM", "Unknown"))
    out["DATE"] = str(source.get("DATE", "0000"))
    out["GENRE"] = str(source.get("GENRE", "Unknown"))
    
    out["TOTALTRACKS"] = total_tracks
    out["TOTALDISCS"] = total_discs

    date_str = out["DATE"]
    yyyy_mm = "0000-00"
    if len(date_str) >= 4:
        yyyy_mm = f"{date_str[:4]}-00"
    
    out["ORIGINAL_YYYY_MM"] = str(source.get("ORIGINAL_YYYY_MM", yyyy_mm))
    out["RELEASE_YYYY_MM"] = str(source.get("RELEASE_YYYY_MM", yyyy_mm))
    
    out["ORIGINAL_YEAR"] = out["ORIGINAL_YYYY_MM"][:4]
    out["RELEASE_YEAR"] = out["RELEASE_YYYY_MM"][:4]
    
    def format_human_date(ym):
        if ym == "0000-00": return "Unknown Date"
        try:
            parts = ym.split("-")
            y = int(parts[0])
            return str(y)
        except:
            return ym

    out["ORIGINAL_DATE"] = format_human_date(out["ORIGINAL_YYYY_MM"])
    out["RELEASE_DATE"] = format_human_date(out["RELEASE_YYYY_MM"])

    out["COUNTRY"] = str(source.get("COUNTRY", ""))
    out["LABEL"] = str(source.get("LABEL", ""))
    out["CATALOGNUMBER"] = str(source.get("CATALOGNUMBER", ""))
    out["MEDIA"] = str(source.get("MEDIA", ""))

    comment_parts = [out["RELEASE_YEAR"], out["COUNTRY"], out["LABEL"], out["CATALOGNUMBER"]]
    generated_comment = " ".join([p for p in comment_parts if p])
    out["COMMENT"] = str(source.get("COMMENT", generated_comment))

    out["CUSTOM_ID"] = str(source.get("CUSTOM_ID", ""))
    out["CUSTOM_ALBUMARTIST"] = str(source.get("CUSTOM_ALBUMARTIST", out["ALBUMARTIST"]))
    out["CUSTOM_STRING"] = str(source.get("CUSTOM_STRING", ""))
    
    out["DISCOGS_URL"] = str(source.get("DISCOGS_URL", ""))
    out["MUSICBRAINZ_URL"] = str(source.get("MUSICBRAINZ_URL", ""))
    out["MUSICBRAINZ_ALBUMID"] = str(source.get("MUSICBRAINZ_ALBUMID", ""))
    out["MUSICBRAINZ_ALBUMARTISTID"] = str(source.get("MUSICBRAINZ_ALBUMARTISTID", ""))
    out["MUSICBRAINZ_RELEASEGROUPID"] = str(source.get("MUSICBRAINZ_RELEASEGROUPID", ""))
    out["ACCURIPID"] = str(source.get("ACCURIPID", ""))
    out["CTDBID"] = str(source.get("CTDBID", ""))
    out["DISCID"] = str(source.get("DISCID", ""))

    out["album_root_path"] = str(source.get("album_root_path", "")) 

    unix_p = source.get("UNIX_ADDED_PRIMARY")
    unix_l = source.get("UNIX_ADDED_LOCAL", "0")
    unix_a = source.get("UNIX_ADDED_APPLEMUSIC", "0")
    unix_y = source.get("UNIX_ADDED_YOUTUBE", "0")
    
    out["UNIX_ADDED_PRIMARY"] = str(unix_p) if unix_p else ""
    out["UNIX_ADDED_LOCAL"] = str(unix_l)
    out["UNIX_ADDED_APPLEMUSIC"] = str(unix_a)
    out["UNIX_ADDED_YOUTUBE"] = str(unix_y)
    
    final_unix = 0
    if unix_p:
         final_unix = int(unix_p)
    else:
        candidates = []
        if unix_l != "0": candidates.append(int(unix_l))
        if unix_a != "0": candidates.append(int(unix_a))
        if unix_y != "0": candidates.append(int(unix_y))
        if candidates:
            final_unix = max(candidates)
            
    out["unix_added"] = final_unix
    if final_unix > 0:
        dt = datetime.datetime.fromtimestamp(final_unix)
        out["date_added"] = dt.strftime("%B %d %Y")
    else:
        out["date_added"] = ""

    return out

def resolve_track_tags(source: dict, index: int) -> dict:
    out = {}
    
    out["TITLE"] = str(source.get("TITLE", "Untitled"))
    out["ARTIST"] = str(source.get("ARTIST", source.get("ALBUMARTIST", "Unknown")))
    
    out["TRACKNUMBER"] = int(source.get("TRACKNUMBER", index + 1))
    out["DISCNUMBER"] = int(source.get("DISCNUMBER", 1))
    
    out["MUSICBRAINZ_ARTISTID"] = str(source.get("MUSICBRAINZ_ARTISTID", ""))
    out["MUSICBRAINZ_TRACKID"] = str(source.get("MUSICBRAINZ_TRACKID", ""))
    out["MUSICBRAINZ_RELEASETRACKID"] = str(source.get("MUSICBRAINZ_RELEASETRACKID", ""))

    return out
