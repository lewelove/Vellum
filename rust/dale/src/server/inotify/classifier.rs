use crate::server::state::AppState;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ChangeFlags {
    pub config: bool,
    pub interfaces_asset: HashSet<String>,
    pub changed_albums: HashSet<PathBuf>,
}

pub async fn classify_events(paths: &[PathBuf], state: &Arc<AppState>) -> ChangeFlags {
    let mut flags = ChangeFlags {
        config: false,
        interfaces_asset: HashSet::new(),
        changed_albums: HashSet::new(),
    };

    let guard = state.config.read().await;
    let config_dir = guard.config_dir.clone();
    let music_directory = guard.music_directory.clone();
    let deps: Vec<PathBuf> = guard.resolved_dependencies.clone();
    let declared_manifests = guard.app.compiler.manifests.clone();

    let mut interface_assets: Vec<(String, PathBuf, bool)> = Vec::new();
    for (name, cfg) in &guard.interfaces {
        for asset_str in cfg.assets.values() {
            let p = libdale::utils::expand_path(asset_str);
            let p = if p.is_absolute() { p } else { config_dir.join(p) };
            if let Ok(canon) = p.canonicalize() {
                interface_assets.push((name.clone(), canon.clone(), canon.is_dir()));
            }
        }
    }
    drop(guard);

    let logic = state.logic.read().await;
    let mut active = state.active_writes.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

    for p in paths {
        let p_canon = p.canonicalize().unwrap_or_else(|_| p.clone());

        if active.remove(&p_canon) || active.remove(p) {
            continue;
        }

        if deps.contains(&p_canon) || p_canon.starts_with(&config_dir) || p.starts_with(&config_dir) {
            flags.config = true;
        }

        if let Some((album_id, album_path)) =
            resolve_album_info(&p_canon, &music_directory, &logic.dict)
                .or_else(|| resolve_album_info(p, &music_directory, &logic.dict))
        {
            if let Ok(mut tracked) = state.tracked_albums.lock() {
                tracked.insert(album_id);
            }
            if is_manifest_file(&p_canon, declared_manifests.as_deref())
                || is_manifest_file(p, declared_manifests.as_deref())
            {
                flags.changed_albums.insert(album_path);
            }
        }

        for (name, asset_path, is_dir) in &interface_assets {
            if *is_dir {
                if p_canon.starts_with(asset_path) {
                    flags.interfaces_asset.insert(name.clone());
                }
            } else if p_canon == *asset_path {
                flags.interfaces_asset.insert(name.clone());
            }
        }
    }
    drop(active);
    drop(logic);

    flags
}

fn is_manifest_file(path: &Path, declared_manifests: Option<&[String]>) -> bool {
    let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
        return false;
    };

    if file_name.eq_ignore_ascii_case("metadata.toml")
        || file_name.eq_ignore_ascii_case("theme.toml")
        || file_name.eq_ignore_ascii_case("album.lock.json")
    {
        return true;
    }

    if let Some(manifests) = declared_manifests {
        let is_toml = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"));
        if !is_toml {
            return false;
        }

        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        for m in manifests {
            let m_clean = m.strip_suffix(".toml").unwrap_or(m);
            if file_stem.eq_ignore_ascii_case(m_clean) {
                return true;
            }
        }
    }

    false
}

fn resolve_album_info(
    path: &Path,
    music_directory: &Path,
    dict: &std::collections::HashMap<String, serde_json::Value>,
) -> Option<(String, PathBuf)> {
    let rel = path.strip_prefix(music_directory).ok()?;
    let mut curr = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    while curr.starts_with(music_directory) && curr != music_directory {
        let Ok(curr_rel) = curr.strip_prefix(music_directory) else {
            break;
        };
        let rel_str = curr_rel.to_string_lossy().to_string();

        if curr.join("metadata.toml").exists()
            || curr.join("album.lock.json").exists()
            || dict.contains_key(&rel_str)
        {
            return Some((rel_str, curr));
        }

        let Some(parent) = curr.parent() else {
            break;
        };
        curr = parent.to_path_buf();
    }

    let rel_str = rel.to_string_lossy().to_string();
    if dict.contains_key(&rel_str) {
        let album_path = music_directory.join(&rel_str);
        return Some((rel_str, album_path));
    }

    None
}
