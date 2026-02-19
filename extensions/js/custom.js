export function resolve_album_tag_custom_id(ctx) {
    return String(ctx.source.CUSTOM_ID || "");
}

export function resolve_album_tag_custom_albumartist(ctx) {
    const candidates = ["CUSTOM_ALBUMARTIST", "ARTISTARTIST", "ALBUMARTIST"];
    for (const key of candidates) {
        if (ctx.source[key]) return String(ctx.source[key]);
    }
    return "Unknown";
}

export function resolve_album_tag_custom_string(ctx) {
    const candidates = ["CUSTOM_STRING", "CUSTOMSTRING"];
    for (const key of candidates) {
        if (ctx.source[key]) return String(ctx.source[key]);
    }
    return "";
}

export function resolve_album_tag_old_comment(ctx) {
    return String(ctx.source.OLD_COMMENT || "");
}
