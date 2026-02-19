export function resolve_album_tag_replaygain_album_gain(ctx) {
    return String(ctx.source.REPLAYGAIN_ALBUM_GAIN || "");
}

export function resolve_album_tag_replaygain_album_peak(ctx) {
    return String(ctx.source.REPLAYGAIN_ALBUM_PEAK || "");
}

export function resolve_track_tag_replaygain_track_gain(ctx) {
    return String(ctx.source.REPLAYGAIN_TRACK_GAIN || "");
}

export function resolve_track_tag_replaygain_track_peak(ctx) {
    return String(ctx.source.REPLAYGAIN_TRACK_PEAK || "");
}
