use anyhow::{Context, Result};
use crate::config::AppConfig;
use crate::expand_path;

pub async fn run(
    query_str: Option<String>,
    playing: bool,
    toml: bool,
    lock: bool,
) -> Result<()> {
    let (config, _, _) = AppConfig::load().context("Failed to load config")?;
    let lib_root = expand_path(&config.storage.library_root)
        .canonicalize()
        .unwrap_or_else(|_| expand_path(&config.storage.library_root));

    let mut target_paths = Vec::new();

    if let Some(q) = query_str {
        let q_trim = q.trim();
        
        // Transparent CLI-to-Server Bridge: We do not expand the shorthand here.
        // The raw DSL string is transmitted directly to the server side logic
        // which evaluates the string against the local DSL engine.
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

        for id in ids {
            target_paths.push(lib_root.join(id));
        }
    } else if playing {
        let playing_album = crate::run::get_playing_album(&config.storage.library_root).await?;
        target_paths.push(playing_album);
    } else {
        anyhow::bail!("No query provided. Use --playing or provide an SQL query.");
    }

    for path in target_paths {
        let final_path = if toml {
            path.join("metadata.toml")
        } else if lock {
            path.join("metadata.lock.json")
        } else {
            path
        };

        println!("{}", final_path.display());
    }

    Ok(())
}
