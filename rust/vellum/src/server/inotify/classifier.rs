use crate::server::state::AppState;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ChangeFlags {
    pub config: bool,
    pub interfaces_asset: HashSet<String>,
}

pub async fn classify_events(paths: &[PathBuf], state: &Arc<AppState>) -> ChangeFlags {
    let mut flags = ChangeFlags {
        config: false,
        interfaces_asset: HashSet::new(),
    };

    let guard = state.config.read().await;
    let config_dir = guard.config_dir.clone();
    let library_root = guard.library_root.clone();
    let deps: Vec<PathBuf> = guard.resolved_dependencies.clone();

    let mut interface_assets: Vec<(String, PathBuf, bool)> = Vec::new();
    for (name, cfg) in &guard.interfaces {
        for asset_str in cfg.assets.values() {
            let p = libvellum::utils::expand_path(asset_str);
            let p = if p.is_absolute() { p } else { config_dir.join(p) };
            if let Ok(canon) = p.canonicalize() {
                interface_assets.push((name.clone(), canon.clone(), canon.is_dir()));
            }
        }
    }
    drop(guard);

    let logic = state.logic.read().await;

    for p in paths {
        let p_canon = p.canonicalize().unwrap_or_else(|_| p.clone());

        if deps.contains(&p_canon) {
            flags.config = true;
        }

        if let Some(album_id) = resolve_album_id(&p_canon, &library_root, &logic.dict)
            .or_else(|| resolve_album_id(p, &library_root, &logic.dict))
            && let Ok(mut tracked) = state.tracked_albums.lock()
        {
            tracked.insert(album_id);
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
    drop(logic);

    flags
}

fn resolve_album_id(
    path: &Path,
    library_root: &Path,
    dict: &std::collections::HashMap<String, serde_json::Value>,
) -> Option<String> {
    let rel = path.strip_prefix(library_root).ok()?;
    let mut curr = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    while curr.starts_with(library_root) && curr != library_root {
        let Ok(curr_rel) = curr.strip_prefix(library_root) else {
            break;
        };
        let rel_str = curr_rel.to_string_lossy().to_string();

        if curr.join("metadata.toml").exists()
            || curr.join("album.lock.json").exists()
            || dict.contains_key(&rel_str)
        {
            return Some(rel_str);
        }

        let Some(parent) = curr.parent() else {
            break;
        };
        curr = parent.to_path_buf();
    }

    let rel_str = rel.to_string_lossy().to_string();
    if dict.contains_key(&rel_str) {
        return Some(rel_str);
    }

    None
}
