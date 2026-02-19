export function resolve_album_tag_country(ctx) {
    return String(ctx.source.COUNTRY || "");
}

export function resolve_album_tag_label(ctx) {
    return String(ctx.source.LABEL || "");
}

export function resolve_album_tag_catalognumber(ctx) {
    return String(ctx.source.CATALOGNUMBER || "");
}

export function resolve_album_tag_media(ctx) {
    return String(ctx.source.MEDIA || "");
}

export function resolve_album_tag_comment(ctx) {
    if (ctx.source.COMMENT) return String(ctx.source.COMMENT);
    
    const country = resolve_album_tag_country(ctx);
    const label = resolve_album_tag_label(ctx);
    const catNo = resolve_album_tag_catalognumber(ctx);

    if (!country && !label && !catNo) return "";

    const yyyy_mm = String(ctx.standard.RELEASE_YYYY_MM || "");
    const parts = [
        yyyy_mm.substring(0, 4),
        country,
        label,
        catNo
    ];
    return parts.filter(Boolean).join(" ").trim();
}

export function resolve_track_tag_ctdbid(ctx) {
    return String(ctx.source.CTDBID || "");
}

export function resolve_track_tag_accuripid(ctx) {
    return String(ctx.source.ACCURIPID || "");
}

export function resolve_track_tag_discid(ctx) {
    return String(ctx.source.DISCID || "");
}
