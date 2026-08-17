use crate::discogs::DiscogsFetcher;
use crate::models::{ActionConfig, UserSelection};
use crate::terminal::prompt_selection;
use anyhow::{Context, Result};
use libactions::payload::ActionPayload;
use reqwest::Client;
use std::fs;

pub async fn execute(payload: &ActionPayload<ActionConfig>) -> Result<()> {
    let force = payload.options.contains("--force") || payload.options.contains("-f");
    let fetcher = DiscogsFetcher::new()?;
    let http_client = Client::new();

    for item in &payload.albums {
        let target_dir = &item.path;
        let album_lock = &item.lock;
        let album_obj = album_lock.get("album").and_then(serde_json::Value::as_object);
        let Some(album_map) = album_obj else {
            continue;
        };

        let album_id = album_map
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        let info_dir = target_dir.join(&payload.config.action.info_dir);
        let master_file_path = info_dir.join(&payload.config.action.filename);

        if master_file_path.exists() && !force {
            let path_disp = master_file_path.display();
            println!("\x1b[33mSkipping {album_id}: {path_disp} already exists\x1b[0m");
            continue;
        }

        let album_title = album_map
            .get("album")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        let album_artist = album_map
            .get("albumartist")
            .or_else(|| album_map.get("artist"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        if album_title.is_empty() || album_artist.is_empty() {
            println!("\x1b[33mSkipping {album_id}: missing album or albumartist metadata\x1b[0m");
            continue;
        }

        println!("\x1b[1;34mSearching Discogs for:\x1b[0m artist=\"{album_artist}\" album=\"{album_title}\"");

        let results = fetcher.search_masters(album_artist, album_title).await?;

        if results.is_empty() {
            println!("\x1b[31mNo Discogs master releases found for {album_artist} - {album_title}\x1b[0m");
            continue;
        }

        let header = format!("{album_artist} - {album_title}");
        let selection = prompt_selection(&header, &results)?;

        match selection {
            UserSelection::Selected(idx) => {
                let selected_master = &results[idx];
                let master_id = selected_master.id;
                println!("\x1b[34mFetching Discogs Master #{master_id}\x1b[0m...");

                let master_data = fetcher.fetch_master_detail(master_id).await?;

                fs::create_dir_all(&info_dir).context("Failed to create Info directory")?;

                let pretty_json = serde_json::to_string_pretty(&master_data)
                    .context("Failed to serialize master JSON")?;

                fs::write(&master_file_path, pretty_json)
                    .context("Failed to write discogs_master.json")?;

                let path_disp = master_file_path.display();
                println!("\x1b[32m✔ Saved raw Discogs master to: {path_disp}\x1b[0m");

                if !album_id.is_empty() {
                    trigger_update(&http_client, album_id).await;
                }
            }
            UserSelection::Skip => {
                println!("\x1b[33mSkipped {album_id}\x1b[0m");
            }
            UserSelection::Quit => {
                println!("\x1b[33mQuit requested. Exiting.\x1b[0m");
                break;
            }
        }
    }

    Ok(())
}

async fn trigger_update(client: &Client, album_id: &str) {
    let encoded_id = urlencoding::encode(album_id);
    let url = format!("http://127.0.0.1:8000/api/update-album/{encoded_id}");
    let _ = client.post(&url).send().await;
}
