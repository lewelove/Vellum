use crate::server::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

async fn resolve_target_ids(
    params: &HashMap<String, String>,
    state: &Arc<AppState>,
    library_root: &std::path::Path,
) -> Vec<String> {
    let mut target_ids = Vec::new();
    let playing = params.get("playing").is_some_and(|v| v == "true");
    let id_arg = params.get("id").cloned();
    let query_arg = params.get("query").cloned();
    let directory_arg = params.get("directory").cloned();
    let recursive_arg = params.get("recursive").cloned();
    let library_arg = params.get("library").is_some_and(|v| v == "true");

    if let Some(q) = query_arg {
        let query_guard = state.query.read().await;
        target_ids = query_guard.query_ids(&q);
    } else if let Some(id) = id_arg {
        if !id.is_empty() {
            target_ids.push(id);
        }
    } else if playing {
        if let Ok(path) = crate::x::get_playing_album(&library_root.to_string_lossy()).await
            && let Ok(rel) = path.strip_prefix(library_root)
        {
            target_ids.push(rel.to_string_lossy().to_string());
        }
    } else if library_arg {
        for entry in walkdir::WalkDir::new(library_root)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_name() == "album.lock.json"
                && let Ok(content) = std::fs::read_to_string(entry.path())
                && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(id) = lock_json.pointer("/album/id").and_then(|v| v.as_str())
            {
                target_ids.push(id.to_string());
            }
        }
    } else if let Some(dir) = recursive_arg {
        for entry in walkdir::WalkDir::new(libvellum::utils::expand_path(&dir))
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_name() == "album.lock.json"
                && let Ok(content) = std::fs::read_to_string(entry.path())
                && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(id) = lock_json.pointer("/album/id").and_then(|v| v.as_str())
            {
                target_ids.push(id.to_string());
            }
        }
    } else if let Some(dir) = directory_arg {
        let p = libvellum::utils::expand_path(&dir).join("album.lock.json");
        if let Ok(content) = std::fs::read_to_string(&p)
            && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(id) = lock_json.pointer("/album/id").and_then(|v| v.as_str())
        {
            target_ids.push(id.to_string());
        }
    }

    target_ids
}

fn run_external_process(
    action_path: &std::path::Path,
    target_ids: &[String],
    library_root: &std::path::Path,
    params: &HashMap<String, String>,
    app_config_json: &serde_json::Value,
    action_config: &serde_json::Value,
    env_vars: &HashMap<String, String>,
) -> Response {
    let mut lock_jsons = Vec::new();
    for target_id in target_ids {
        let lock_file_path = library_root.join(target_id).join("album.lock.json");
        if let Ok(json_data) = std::fs::read_to_string(&lock_file_path)
            && let Ok(lock_json) = serde_json::from_str::<serde_json::Value>(&json_data)
        {
            lock_jsons.push(lock_json);
        }
    }

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
        "albums": lock_jsons,
        "config": {
            "vellum": app_config_json,
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
    Path(name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let name_key = name.replace('-', "_");

    let config_guard = state.config.read().await;
    let action_cfg_opt = config_guard.actions.get(&name_key).cloned();
    let library_root = config_guard.library_root.clone();
    let app_config_json = serde_json::to_value(&config_guard.app).unwrap_or_else(|_| json!({}));
    let config_dir = config_guard.config_dir.clone();
    let env_vars =
        crate::x::load_env_vars_from_path(config_guard.app.storage.environment.as_deref());
    drop(config_guard);

    let action_cfg = action_cfg_opt.unwrap_or_default();
    let target_ids = resolve_target_ids(&params, &state, &library_root).await;

    if let Some(run_str) = &action_cfg.run {
        let expanded_action_path = libvellum::utils::expand_path(run_str);
        let action_path = if expanded_action_path.is_absolute() {
            expanded_action_path
        } else {
            config_dir.join(expanded_action_path)
        };

        if action_path.exists() {
            return run_external_process(
                &action_path,
                &target_ids,
                &library_root,
                &params,
                &app_config_json,
                &action_cfg.config,
                &env_vars,
            );
        }
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
    if name_key == "open_config_in_terminal" || target_ids.is_empty() {
        let dummy_path = std::path::Path::new("");
        if matches!(
            crate::x::builtin::execute_builtin(&name_key, dummy_path, &merged_config),
            Ok(true)
        ) {
            executed = true;
        }
    } else {
        for target_id in &target_ids {
            let album_path = library_root.join(target_id);
            if matches!(
                crate::x::builtin::execute_builtin(&name_key, &album_path, &merged_config),
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
