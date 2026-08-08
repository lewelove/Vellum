use libdale::utils::expand_path;
use std::path::{Path, PathBuf};

pub struct PreparedContext {
    pub audio_files: Vec<PathBuf>,
    pub music_directory: PathBuf,
}

pub fn prepare_build_context(
    config: &libdale::lua::ResolvedConfig,
    album_root: &Path,
) -> PreparedContext {
    let exts: Vec<String> = config.app.manifest.audio_files.clone().unwrap_or_else(|| vec![".flac".to_string()]);
    let ext_refs: Vec<&str> = exts.iter().map(AsRef::as_ref).collect();
    let audio_files = libdale::scanner::scan_audio_files(album_root, &ext_refs);

    let music_dir_raw = &config.app.storage.music_directory;
    let music_directory = expand_path(music_dir_raw)
        .canonicalize()
        .unwrap_or_else(|_| expand_path(music_dir_raw));

    PreparedContext { audio_files, music_directory }
}
