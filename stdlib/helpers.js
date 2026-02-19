import fs from "node:fs";
import path from "node:path";

const formatTime = (ms) => {
  if (!ms) return "0:00";
  const seconds = Math.floor(ms / 1000);
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  return `${m}:${String(s).padStart(2, '0')}`;
};

export const albumHelpers = {
  total_tracks: (ctx) => String(ctx.tracks.length),
  
  total_discs: (ctx) => String(new Set(ctx.tracks.map(t => t.DISCNUMBER || "1")).size),
  
  album_root_path: (ctx) => {
    const libRoot = ctx.config.storage.library_root;
    return path.relative(libRoot, ctx.album_root);
  },

  metadata_toml_hash: (ctx) => ctx.metadata_toml_hash || "",
  
  metadata_toml_mtime: (ctx) => ctx.metadata_toml_mtime || 0,

  unix_added: (ctx) => {
    const priority = [
      "UNIX_ADDED_PRIMARY", "UNIX_ADDED_APPLEMUSIC", "UNIX_ADDED_YOUTUBE",
      "UNIX_ADDED_FOOBAR", "UNIX_ADDED_LOCAL", "UNIXTIMEAPPLE",
      "UNIXTIMEYOUTUBE", "UNIXTIMEFOOBAR"
    ];
    for (const key of priority) {
      const val = ctx.source[key];
      if (val) {
        const num = parseInt(val, 10);
        if (!isNaN(num)) return num;
      }
    }
    return 0;
  },

  date_added: (ctx) => {
    const unix = albumHelpers.unix_added(ctx);
    if (unix <= 0) return "";
    const date = new Date(unix * 1000);
    const months = [
      "January", "February", "March", "April", "May", "June",
      "July", "August", "September", "October", "November", "December"
    ];
    return `${months[date.getMonth()]} ${date.getDate().toString().padStart(2, '0')} ${date.getFullYear()}`;
  },

  album_duration_in_ms: (ctx) => 
    ctx.tracks.reduce((acc, t) => acc + (parseInt(t.track_duration_in_ms, 10) || 0), 0),

  album_duration_time: (ctx) => formatTime(albumHelpers.album_duration_in_ms(ctx)),

  cover_path: (ctx) => {
    const priorities = ["cover.png", "cover.jpg", "folder.jpg", "folder.png", "front.jpg"];
    for (const p of priorities) {
      if (fs.existsSync(path.join(ctx.album_root, p))) return p;
    }
    
    try {
      const files = fs.readdirSync(ctx.album_root);
      for (const f of files) {
        const ext = path.extname(f).toLowerCase();
        if ([".jpg", ".jpeg", ".png"].includes(ext)) {
          const base = f.toLowerCase();
          if (base.includes("cover") || base.includes("front")) return f;
        }
      }
    } catch (e) {}
    
    return "default_cover.png";
  },

  cover_hash: (ctx) => ctx.cover_hash || "",

  cover_byte_size: (ctx) => {
    const cp = albumHelpers.cover_path(ctx);
    if (!cp || cp === "default_cover.png") return 0;
    try {
      return fs.statSync(path.join(ctx.album_root, cp)).size;
    } catch (e) { return 0; }
  },

  cover_mtime: (ctx) => {
    const cp = albumHelpers.cover_path(ctx);
    if (!cp || cp === "default_cover.png") return 0;
    try {
      return Math.floor(fs.statSync(path.join(ctx.album_root, cp)).mtimeMs / 1000);
    } catch (e) { return 0; }
  }
};

export const trackHelpers = {
  track_path: (ctx) => ctx.track_path || "",

  track_library_path: (ctx) => {
    const libRoot = ctx.config.storage.library_root;
    const abs = path.join(ctx.album_root, ctx.track_path || "");
    return path.relative(libRoot, abs);
  },

  track_mtime: (ctx) => ctx.physics.mtime || 0,
  
  track_size: (ctx) => ctx.physics.file_size || 0,

  lyrics_path: (ctx) => {
    if (ctx.source.lyrics_path) return String(ctx.source.lyrics_path);
    if (ctx.source.LYRICS) return "<METADATA>";

    const relPath = ctx.track_path;
    if (!relPath) return "";
    
    const trackFile = path.basename(relPath);
    const trackRelDir = path.dirname(relPath);
    const stem = path.parse(trackFile).name;

    const directories = [
      path.join(ctx.album_root, "lyrics"),
      path.join(ctx.album_root, trackRelDir)
    ];

    for (const dir of directories) {
      if (!fs.existsSync(dir)) continue;
      for (const ext of [".lrc", ".txt"]) {
        const cand = path.join(dir, stem + ext);
        if (fs.existsSync(cand)) return path.relative(ctx.album_root, cand);
      }
    }
    return "";
  },

  encoding: (ctx) => (ctx.physics.format || "UNKNOWN").toUpperCase(),
  
  bits_per_sample: (ctx) => ctx.physics.bit_depth || 0,
  
  channels: (ctx) => ctx.physics.channels || 0,
  
  sample_rate: (ctx) => ctx.physics.sample_rate || 0,

  track_duration_in_ms: (ctx) => ctx.physics.duration_ms || 0,

  track_duration_time: (ctx) => formatTime(ctx.physics.duration_ms || 0),

  track_duration_in_samples: (ctx) => {
    const sr = ctx.physics.sample_rate || 0;
    const ms = ctx.physics.duration_ms || 0;
    return Math.floor((ms / 1000) * sr);
  }
};
