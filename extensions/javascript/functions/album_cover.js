import { $ } from "bun";
import path from "node:path";

export async function resolve_album_helper_cover_entropy(ctx) {
    const cHash = ctx.paths.cover_hash;
    if (!cHash) return 0;

    const thumbDir = ctx.config.storage.thumbnail_cache_folder;
    if (!thumbDir) return 0;

    const thumbFile = path.join(thumbDir, `${cHash}.png`);
    
    try {
        const output = await $`magick ${thumbFile} -format "%[entropy]" info:`.text();
        return parseFloat(output) || 0;
    } catch (e) {
        return 0;
    }
}

export async function resolve_album_helper_cover_chroma(ctx) {
    const cHash = ctx.cover_hash;
    if (!cHash) return 0;

    const thumbDir = ctx.config.storage.thumbnail_cache_folder;
    if (!thumbDir) return 0;

    const thumbFile = path.join(thumbDir, `${cHash}.png`);

    try {
        const output = await $`magick ${thumbFile} -colorspace HSL -channel Saturation -separate -format "%[mean]" info:`.text();
        return parseFloat(output) || 0;
    } catch (e) {
        return 0;
    }
}
