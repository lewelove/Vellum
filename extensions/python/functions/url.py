def resolve_album_tag_discogs_url(ctx):
    return str(ctx["source"].get("DISCOGS_URL", ""))

def resolve_album_tag_musicbrainz_url(ctx):
    return str(ctx["source"].get("MUSICBRAINZ_URL", ""))
