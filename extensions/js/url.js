export function resolve_album_tag_discogs_url(ctx) {
    return String(ctx.source.DISCOGS_URL || "");
}

export function resolve_album_tag_musicbrainz_url(ctx) {
    return String(ctx.source.MUSICBRAINZ_URL || "");
}
