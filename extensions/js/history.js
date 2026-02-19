export function resolve_album_tag_unix_added_primary(ctx) {
    return String(ctx.source.UNIX_ADDED_PRIMARY || "");
}

export function resolve_album_tag_unix_added_local(ctx) {
    const candidates = ["UNIX_ADDED_LOCAL", "UNIX_ADDED_PRIMARY"];
    for (const key of candidates) {
        if (ctx.source[key]) return String(ctx.source[key]);
    }
    return "";
}

export function resolve_album_tag_unix_added_foobar(ctx) {
    const candidates = ["UNIX_ADDED_FOOBAR", "UNIXTIMEFOOBAR"];
    for (const key of candidates) {
        if (ctx.source[key]) return String(ctx.source[key]);
    }
    return "";
}

export function resolve_album_tag_unix_added_applemusic(ctx) {
    const candidates = ["UNIX_ADDED_APPLEMUSIC", "UNIXTIMEAPPLE"];
    for (const key of candidates) {
        if (ctx.source[key]) return String(ctx.source[key]);
    }
    return "";
}

export function resolve_album_tag_unix_added_youtube(ctx) {
    const candidates = ["UNIX_ADDED_YOUTUBE", "UNIXTIMEYOUTUBE"];
    for (const key of candidates) {
        if (ctx.source[key]) return String(ctx.source[key]);
    }
    return "";
}
