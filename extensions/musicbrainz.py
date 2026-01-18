def resolve_album_tag_musicbrainz_albumartistid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ALBUMARTISTID", ""))

def resolve_album_tag_musicbrainz_releasegroupid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_RELEASEGROUPID", ""))

def resolve_album_tag_musicbrainz_albumid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ALBUMID", ""))

def resolve_track_tag_musicbrainz_artistid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_ARTISTID", ""))

def resolve_track_tag_musicbrainz_releasetrackid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_RELEASETRACKID", ""))

def resolve_track_tag_musicbrainz_trackid(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_TRACKID", ""))
