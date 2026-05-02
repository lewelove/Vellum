use anyhow::{Context, Result};
use crate::config::AppConfig;
use crate::expand_path;

pub async fn run(
    query_str: Option<String>,
    playing: bool,
    toml: bool,
    lock: bool,
    raw: bool,
    id_flag: bool,
) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    let lib_root = expand_path(&config.storage.library_root)
        .canonicalize()
        .unwrap_or_else(|_| expand_path(&config.storage.library_root));

    let mut target_ids = Vec::new();

    if let Some(q) = query_str {
        let q_trim = q.trim();
        let full_sql = if q_trim.is_empty() {
            "SELECT id FROM albums".to_string()
        } else {
            let upper_q = q_trim.to_uppercase();
            if upper_q.starts_with("SELECT") {
                q_trim.to_string()
            } else {
                let prefix = if upper_q.starts_with("WHERE")
                    || upper_q.starts_with("ORDER")
                    || upper_q.starts_with("LIMIT")
                {
                    "SELECT id FROM albums "
                } else {
                    "SELECT id FROM albums WHERE "
                };
                format!("{}{}", prefix, q_trim)
            }
        };

        let client = reqwest::Client::new();
        let res = client
            .post("http://127.0.0.1:8000/api/internal/query")
            .json(&serde_json::json!({ "query": full_sql }))
            .send()
            .await
            .context("Failed to connect to the Vellum server. Is it running?")?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            anyhow::bail!("Server rejected query: {}", err_text);
        }

        let ids: Vec<String> = res.json().await.context("Invalid response from server")?;
        target_ids = ids;
    } else if playing {
        let playing_path = crate::run::get_playing_album(&config.storage.library_root).await?;
        let rel_path = playing_path.strip_prefix(&lib_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| playing_path.to_string_lossy().to_string());
        target_ids.push(rel_path);
    } else {
        anyhow::bail!("No query provided. Use --playing or provide an SQL query.");
    }

    for id in target_ids {
        if raw {
            let lock_file_path = lib_root.join(&id).join("metadata.lock.json");
            if let Ok(content) = std::fs::read_to_string(&lock_file_path) {
                println!("{}", content);
            }
        } else if id_flag {
            println!("{}", id);
        } else {
            let base_path = lib_root.join(&id);
            let final_path = if toml {
                base_path.join("metadata.toml")
            } else if lock {
                base_path.join("metadata.lock.json")
            } else {
                base_path
            };
            println!("{}", final_path.display());
        }
    }

    Ok(())
}
