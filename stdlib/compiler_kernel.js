import fs from "node:fs";
import path from "node:path";

async function main() {
  const rawInput = fs.readFileSync(0, "utf-8");
  if (!rawInput) process.exit(0);

  const manifest = JSON.parse(rawInput);
  const { config, metadata, harvest, standard, paths } = manifest;

  const albumSource = metadata.album || {};
  const trackEntries = metadata.tracks || [];

  const registry = await loadExtensionRegistry(config, paths.project_root);

  const finalTracks = [];
  for (let idx = 0; idx < harvest.length; idx++) {
    const file = harvest[idx];
    const entry = trackEntries[idx] || {};
    const stdTrack = standard.tracks[idx] || {};
    
    const ctx = {
      source: { ...albumSource, ...entry },
      physics: file.physics,
      raw_tags: file.tags,
      standard: stdTrack,
      album_root: paths.album_root,
      project_root: paths.project_root,
      track_path: file.track_path,
      cover_hash: paths.cover_hash,
      config
    };

    const result = {};
    const layoutKeys = getLayoutKeys(config.lock.layout.tracks);
    
    for (const key of layoutKeys) {
      const resolver = registry.trackTags[key] || registry.trackHelpers[key.toLowerCase()];
      if (resolver) {
        result[key] = await resolver(ctx);
      } else {
        result[key] = stdTrack[key] ?? (ctx.source[key] || "");
      }
    }
    
    result.track_path = ctx.track_path;
    finalTracks.push(result);
  }

  const albumCtx = {
    source: albumSource,
    tracks: finalTracks,
    standard: standard.album,
    album_root: paths.album_root,
    project_root: paths.project_root,
    metadata_toml_hash: paths.metadata_toml_hash,
    metadata_toml_mtime: paths.metadata_toml_mtime,
    cover_hash: paths.cover_hash,
    config
  };

  const finalAlbum = {};
  const albumLayoutKeys = getLayoutKeys(config.lock.layout.album);
  
  for (const key of albumLayoutKeys) {
    const resolver = registry.albumTags[key] || registry.albumHelpers[key.toLowerCase()];
    if (resolver) {
      finalAlbum[key] = await resolver(albumCtx);
    } else {
      finalAlbum[key] = standard.album[key] ?? (albumSource[key] || "");
    }
  }

  process.stdout.write(JSON.stringify({ album: finalAlbum, tracks: finalTracks }));
}

async function loadExtensionRegistry(config, projectRoot) {
  const reg = {
    albumTags: {},
    albumHelpers: {},
    trackTags: {},
    trackHelpers: {}
  };

  const extensions = config.compiler?.extensions || {};
  // Point specifically to the js subdirectory within the project root
  const extDir = path.join(projectRoot, "extensions", "js");

  for (const [filename, allowedKeys] of Object.entries(extensions)) {
    try {
      const extPath = path.join(extDir, `${filename}.js`);
      if (!fs.existsSync(extPath)) continue;

      const mod = await import(extPath);
      for (const [funcName, func] of Object.entries(mod)) {
        if (typeof func !== 'function') continue;

        const parts = funcName.split("_");
        if (parts[0] !== "resolve" || parts.length < 4) continue;

        const scope = parts[1];
        const kind = parts[2];
        const keyRaw = parts.slice(3).join("_");
        
        const isAllowed = allowedKeys.some(k => 
          k === keyRaw || k === keyRaw.toUpperCase() || k === keyRaw.toLowerCase()
        );

        if (!isAllowed) continue;

        if (scope === "album" && kind === "tag") reg.albumTags[keyRaw.toUpperCase()] = func;
        if (scope === "album" && kind === "helper") reg.albumHelpers[keyRaw.toLowerCase()] = func;
        if (scope === "track" && kind === "tag") reg.trackTags[keyRaw.toUpperCase()] = func;
        if (scope === "track" && kind === "helper") reg.trackHelpers[keyRaw.toLowerCase()] = func;
      }
    } catch (e) {
      console.error(`Extension Error [${filename}]:`, e);
    }
  }
  return reg;
}

function getLayoutKeys(layout) {
  const keys = new Set();
  const walk = (items) => {
    for (const item of items) {
      if (typeof item === "string") {
        if (!item.startsWith("#") && !["\n", "*"].includes(item)) {
          keys.add(item);
        }
      } else if (typeof item === "object") {
        Object.values(item).forEach(walk);
      }
    }
  };
  if (layout) walk(layout);
  return keys;
}

main();
