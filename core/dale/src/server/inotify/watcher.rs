use crate::server::inotify::classifier;
use crate::server::inotify::handler;
use crate::server::state::AppState;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub fn setup_watcher(tx: UnboundedSender<Vec<PathBuf>>) -> RecommendedWatcher {
    RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res
                && (event.kind.is_modify()
                    || event.kind.is_create()
                    || event.kind.is_remove())
            {
                let _ = tx.send(event.paths);
            }
        },
        notify::Config::default(),
    )
    .expect("Failed to create inotify watcher")
}

pub async fn run_loop(
    mut rx: UnboundedReceiver<Vec<PathBuf>>,
    watcher: &mut RecommendedWatcher,
    state: Arc<AppState>,
) {
    let config_dir = {
        let guard = state.config.read().await;
        guard.config_dir.clone()
    };

    let mut watched_dirs = std::collections::HashSet::new();

    sync_watches(watcher, &mut watched_dirs, &state, &config_dir).await;

    while let Some(mut paths) = rx.recv().await {
        tokio::time::sleep(Duration::from_millis(100)).await;
        while let Ok(more_paths) = rx.try_recv() {
            paths.extend(more_paths);
        }

        paths.sort();
        paths.dedup();

        let flags = classifier::classify_events(&paths, &state).await;
        handler::process_events(flags, &state).await;

        sync_watches(watcher, &mut watched_dirs, &state, &config_dir).await;
    }
}

fn collect_asset_watches(
    config_dir: &Path,
    assets: impl Iterator<Item = impl AsRef<str>>,
    recursive: &mut HashSet<PathBuf>,
    non_recursive: &mut HashSet<PathBuf>,
) {
    for asset_str in assets {
        let p = libdale::utils::expand_path(asset_str.as_ref());
        let p = if p.is_absolute() {
            p
        } else {
            config_dir.join(p)
        };
        let Ok(canon) = p.canonicalize() else {
            continue;
        };
        if canon.is_dir() {
            recursive.insert(canon);
        } else if canon.is_file()
            && let Some(parent) = canon.parent()
        {
            non_recursive.insert(parent.to_path_buf());
        }
    }
}

fn apply_watch_set(
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    dirs: HashSet<PathBuf>,
    mode: RecursiveMode,
) {
    for dir in dirs {
        if dir.exists() && watched_dirs.insert(dir.clone()) {
            let _ = watcher.watch(&dir, mode);
        }
    }
}

async fn sync_watches(
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut std::collections::HashSet<PathBuf>,
    state: &Arc<AppState>,
    config_dir: &Path,
) {
    let guard = state.config.read().await;

    let mut needed_recursive = std::collections::HashSet::new();
    let mut needed_non_recursive = std::collections::HashSet::new();

    needed_recursive.insert(config_dir.to_path_buf());
    needed_recursive.insert(guard.music_directory.clone());

    for cfg in guard.interfaces.values() {
        collect_asset_watches(
            config_dir,
            cfg.assets.values(),
            &mut needed_recursive,
            &mut needed_non_recursive,
        );
    }

    for dep in &guard.resolved_dependencies {
        if let Some(parent) = dep.parent() {
            needed_non_recursive.insert(parent.to_path_buf());
        }
    }

    let deps_graph = state.deps_graph.read().await;
    for dep_file in deps_graph.file_to_albums.keys() {
        if !dep_file.starts_with(config_dir)
            && !dep_file.starts_with(&guard.music_directory)
            && let Some(parent) = dep_file.parent()
        {
            needed_non_recursive.insert(parent.to_path_buf());
        }
    }
    drop(deps_graph);
    drop(guard);

    apply_watch_set(
        watcher,
        watched_dirs,
        needed_recursive,
        RecursiveMode::Recursive,
    );
    apply_watch_set(
        watcher,
        watched_dirs,
        needed_non_recursive,
        RecursiveMode::NonRecursive,
    );
}
