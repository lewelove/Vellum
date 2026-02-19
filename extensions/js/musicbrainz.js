export function resolve_album_tag_musicbrainz_albumartistid(ctx) {
    return String(ctx.source.MUSICBRAINZ_ALBUMARTISTID || "");
}

export function resolve_album_tag_musicbrainz_releasegroupid(ctx) {
    return String(ctx.source.MUSICBRAINZ_RELEASEGROUPID || "");
}

export function resolve_album_tag_musicbrainz_albumid(ctx) {
    return String(ctx.source.MUSICBRAINZ_ALBUMID || "");
}

export function resolve_track_tag_musicbrainz_artistid(ctx) {
    return String(ctx.source.MUSICBRAINZ_ARTISTID || "");
}

export function resolve_track_tag_musicbrainz_releasetrackid(ctx) {
    return String(ctx.source.MUSICBRAINZ_RELEASETRACKID || "");
}

export function resolve_track_tag_musicbrainz_trackid(ctx) {
    return String(ctx.source.MUSICBRAINZ_TRACKID || "");
}
