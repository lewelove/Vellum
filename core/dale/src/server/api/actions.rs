use crate::server::state::AppState;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct TargetAlbumEntry {
    path: PathBuf,
    lock: serde_json::Value,
}

async fn resolve_target_entries(
    params: &HashMap<String, String>,
    state: &Arc<AppState>,
) -> Vec<TargetAlbumEntry> {
    let playing = params.get("playing").is_some_and(|v| v == "true");
    let id_arg = params.get("id").cloned();
    let query_arg = params.get("query").cloned();
    let directory_arg = params.get("directory").cloned();
    let recursive_arg = params.get("recursive").cloned();
    let library_arg = params.get("library").is_some_and(|v| v == "true");

    let mut target_ids = Vec::new();

    if playing {
        let music_dir = state.config.read().await.music_directory.clone();
        if let Ok(playing_path) = crate::x::get_playing_track_url().await {
            let clean_playing = playing_path.trim_start_matches('/');
            let logic_guard = state.logic.read().await;
            if let Some(id) = logic_guard
                .path_lookup
                .get(clean_playing)
                .or_else(|| logic_guard.path_lookup.get(&playing_path))
            {
                target_ids.push(id.clone());
            } else if let Ok(dir) = crate::x::get_playing_album(&music_dir.to_string_lossy()).await {
                let canon = dir.canonicalize().unwrap_or(dir);
                if let Some(id) = logic_guard.albums_by_path.get(&canon) {
                    target_ids.push(id.clone());
                }
            }
        }
    } else {
        let logic_guard = state.logic.read().await;
        if let Some(q) = query_arg {
            target_ids = logic_guard.find_ids(&q);
        } else if let Some(id) = id_arg {
            if !id.is_empty() {
                target_ids.push(id);
            }
        } else if library_arg || recursive_arg.is_some() {
            let music_dir = state.config.read().await.music_directory.clone();
            let root = recursive_arg
                .map_or_else(|| music_dir, |dir| libdale::utils::expand_path(&dir));
            let root_canon = root.canonicalize().unwrap_or(root);
            for (album_path, id) in &logic_guard.albums_by_path {
                if album_path.starts_with(&root_canon) {
                    target_ids.push(id.clone());
                }
            }
        } else if let Some(dir) = directory_arg {
            let p = libdale::utils::expand_path(&dir);
            let p_canon = p.canonicalize().unwrap_or(p);
            if let Some(id) = logic_guard.albums_by_path.get(&p_canon) {
                target_ids.push(id.clone());
            }
        }
    }

    let logic = state.logic.read().await;
    let mut list = Vec::new();
    for target_id in target_ids {
        if let Some(path) = logic.path_by_id.get(&target_id)
            && let Some(json_str) = logic.get_album_json(&target_id)
            && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&json_str)
        {
            list.push(TargetAlbumEntry {
                path: path.clone(),
                lock: lock_json,
            });
        }
    }
    list
}

fn run_external_process(
    action_path: &Path,
    target_entries: Vec<TargetAlbumEntry>,
    params: &HashMap<String, String>,
    app_config_json: &serde_json::Value,
    action_config: &serde_json::Value,
    env_vars: HashMap<String, String>,
) -> Response {
    let album_entries: Vec<serde_json::Value> = target_entries
        .into_iter()
        .map(|e| {
            json!({
                "path": e.path.to_string_lossy(),
                "lock": e.lock
            })
        })
        .collect();

    let mut options_vec = Vec::new();
    for (k, v) in params {
        if k != "playing"
            && k != "id"
            && k != "query"
            && k != "directory"
            && k != "recursive"
            && k != "library"
        {
            if v.is_empty() {
                options_vec.push(k.clone());
            } else {
                options_vec.push(format!("{k}={v}"));
            }
        }
    }
    let options_str = options_vec.join(" ");

    let combined_json = json!({
        "albums": album_entries,
        "config": {
            "dale": app_config_json,
            "action": action_config
        },
        "options": options_str
    });

    let cmd = if action_path.extension().is_some_and(|e| e == "py") {
        "python"
    } else if action_path.extension().is_some_and(|e| e == "sh") {
        "sh"
    } else {
        action_path.to_str().unwrap()
    };

    let mut command = tokio::process::Command::new(cmd);
    command.envs(env_vars);
    if cmd == "python" || cmd == "sh" {
        command.arg(action_path);
    }

    match command.stdin(std::process::Stdio::piped()).spawn() {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let payload = serde_json::to_string(&combined_json).unwrap_or_default();
                tokio::spawn(async move {
                    let _ =
                        tokio::io::AsyncWriteExt::write_all(&mut stdin, payload.as_bytes()).await;
                });
            }
            tokio::spawn(async move {
                if let Ok(status) = child.wait().await {
                    if !status.success() {
                        log::error!("Action failed with status: {status}");
                    }
                } else {
                    log::error!("Failed to wait on action child process.");
                }
            });
            Json(json!({"status": "ok"})).into_response()
        }
        Err(e) => {
            log::error!("Failed to spawn action: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn execute_action(
    axum::extract::Path(name): axum::extract::Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let name_key = name.replace('-', "_");

    let config_guard = state.config.read().await;
    let action_cfg_opt = config_guard.actions.get(&name_key).cloned();
    let app_config_json = serde_json::to_value(&config_guard.app).unwrap_or_else(|_| json!({}));
    let env_vars =
        crate::x::load_env_vars_from_path(config_guard.app.storage.environment.as_deref());
    drop(config_guard);

    let action_cfg = action_cfg_opt.unwrap_or_default();
    let target_entries = resolve_target_entries(&params, &state).await;

    if let Some(run_str) = &action_cfg.run {
        let action_path = PathBuf::from(run_str);

        if tokio::fs::try_exists(&action_path).await.unwrap_or(false) {
            return run_external_process(
                &action_path,
                target_entries,
                &params,
                &app_config_json,
                &action_cfg.config,
                env_vars,
            );
        }
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Action '{name}' script not found at path: {}", action_path.display())})),
        )
            .into_response();
    }

    let mut merged_config = action_cfg.config.clone();
    if let serde_json::Value::Object(ref mut map) = merged_config {
        for (k, v) in &params {
            if !["id", "playing", "query", "directory", "recursive", "library"].contains(&k.as_str())
            {
                map.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
        }
    }

    let mut executed = false;
    if name_key == "open_config_in_terminal" || target_entries.is_empty() {
        let dummy_path = Path::new("");
        if matches!(
            crate::x::builtin::execute_builtin(&name_key, dummy_path, &merged_config),
            Ok(true)
        ) {
            executed = true;
        }
    } else {
        for entry in &target_entries {
            if matches!(
                crate::x::builtin::execute_builtin(&name_key, &entry.path, &merged_config),
                Ok(true)
            ) {
                executed = true;
            }
        }
    }

    if executed {
        Json(json!({"status": "ok"})).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Action '{name}' is not recognized or configured.")})),
        )
            .into_response()
    }
}
