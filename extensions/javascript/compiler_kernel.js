import fs from "node:fs";
import path from "node:path";

/**
 * Logic: The JavaScript Kernel acts as an enrichment layer.
 * 1. Receives a pre-populated Intermediary Object from Rust.
 * 2. Scans the extensions folder for matching resolve functions.
 * 3. Maps specific source and standard context for each scope using immutable copies.
 * 4. Executes the extension and overrides the Rust-provided value.
 */

let registry = null;

async function main() {
  const stdinIterator = Bun.stdin.stream().getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await stdinIterator.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    let lineEndIndex;
    
    while ((lineEndIndex = buffer.indexOf('\n')) !== -1) {
      const line = buffer.slice(0, lineEndIndex).trim();
      buffer = buffer.slice(lineEndIndex + 1);

      if (line) {
        const manifest = JSON.parse(line);
        const baseCtx = manifest.ctx;

        if (!registry) {
          const extCfg = baseCtx.config.extensions || {};
          const functionsDir = path.resolve(extCfg.folder, extCfg.functions_folder || "functions");
          registry = await loadExtensionRegistry(functionsDir);
        }

        const albumData = manifest.album;
        const tracksData = manifest.tracks;

        // Enrich Album
        // Use a scoped context to avoid mutating baseCtx
        const albumCtx = { 
          ...baseCtx,
          source: baseCtx.metadata.album || {},
          standard: baseCtx.standard.album || {}
        };

        for (const key of Object.keys(albumData)) {
          const resolver = registry.albumTags[key] || registry.albumHelpers[key.toLowerCase()];
          if (resolver) {
            albumData[key] = await resolver(albumCtx);
          }
        }

        // Enrich Tracks
        for (let i = 0; i < tracksData.length; i++) {
          const track = tracksData[i];
          
          // Construct track-specific context using the original baseCtx
          const tCtx = { 
            ...baseCtx, 
            source: { ... (baseCtx.metadata.album || {}), ...(baseCtx.metadata.tracks[i] || {}) },
            standard: baseCtx.standard.tracks[i] || {},
            harvest_item: baseCtx.harvest[i] || {} 
          };
          
          for (const key of Object.keys(track)) {
            const resolver = registry.trackTags[key] || registry.trackHelpers[key.toLowerCase()];
            if (resolver) {
              track[key] = await resolver(tCtx);
            }
          }
        }

        // Return the enriched data with the original context for path resolution
        process.stdout.write(JSON.stringify({ album: albumData, tracks: tracksData, ctx: baseCtx }) + "\n");
      }
    }
  }
}

async function loadExtensionRegistry(extDir) {
  const reg = { albumTags: {}, albumHelpers: {}, trackTags: {}, trackHelpers: {} };
  if (!fs.existsSync(extDir)) return reg;

  const files = fs.readdirSync(extDir).filter(f => f.endsWith(".js"));
  for (const filename of files) {
    const mod = await import(path.join(extDir, filename));
    for (const [funcName, func] of Object.entries(mod)) {
      if (typeof func !== 'function' || !funcName.startsWith("resolve_")) continue;
      const parts = funcName.split("_");
      const scope = parts[1];
      const kind = parts[2];
      const keyRaw = parts.slice(3).join("_");

      if (scope === "album") {
        if (kind === "tag") reg.albumTags[keyRaw.toUpperCase()] = func;
        else reg.albumHelpers[keyRaw.toLowerCase()] = func;
      } else if (scope === "track") {
        if (kind === "tag") reg.trackTags[keyRaw.toUpperCase()] = func;
        else reg.trackHelpers[keyRaw.toLowerCase()] = func;
      }
    }
  }
  return reg;
}

main();
