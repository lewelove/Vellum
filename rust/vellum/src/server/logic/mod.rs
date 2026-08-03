pub mod cache;
pub mod query;
pub mod sort;

pub use sort::{SortKey, value_to_sort_key};

use anyhow::Result;
use libvellum::lua::{LogicManifest, get_or_init_lua_vm, reset_lua_vm};
use roaring::RoaringBitmap;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

pub struct LogicEngine {
    pub manifest: LogicManifest,
    pub(crate) libraries_cache: HashMap<String, RoaringBitmap>,
    pub(crate) filters_cache: HashMap<String, RoaringBitmap>,
    pub(crate) facets_cache: HashMap<String, HashMap<String, RoaringBitmap>>,
    pub(crate) orders_cache: HashMap<String, Vec<u32>>,
    pub(crate) shelves_cache: HashMap<String, Vec<u32>>,
    pub(crate) uid_to_id: HashMap<u32, String>,
    pub(crate) id_to_uid: HashMap<String, u32>,
    pub(crate) next_uid: u32,
    pub dict: HashMap<String, Value>,
    pub track_lookup: HashMap<String, Value>,
    pub path_lookup: HashMap<String, String>,
    pub(crate) evaluated_logic: HashMap<u32, Value>,
    pub(crate) lock_cache: HashMap<String, String>,
}

impl LogicEngine {
    pub fn new() -> Result<Self> {
        let config_path = libvellum::lua::resolve_config_path().unwrap_or_default();
        let manifest = get_or_init_lua_vm(&config_path, |engine| {
            let eval = engine.evaluate_config(&config_path)?;
            Ok(eval.manifest)
        })?;

        Ok(Self {
            manifest,
            libraries_cache: HashMap::new(),
            filters_cache: HashMap::new(),
            facets_cache: HashMap::new(),
            orders_cache: HashMap::new(),
            shelves_cache: HashMap::new(),
            uid_to_id: HashMap::new(),
            id_to_uid: HashMap::new(),
            next_uid: 1,
            dict: HashMap::new(),
            track_lookup: HashMap::new(),
            path_lookup: HashMap::new(),
            evaluated_logic: HashMap::new(),
            lock_cache: HashMap::new(),
        })
    }

    pub fn reload_manifest(&mut self, config_path: &Path) -> Result<()> {
        reset_lua_vm();
        let manifest = get_or_init_lua_vm(config_path, |engine| {
            let eval = engine.evaluate_config(config_path)?;
            let mut new_evaluated = HashMap::new();
            for (&uid, id) in &self.uid_to_id {
                if let Some(json_str) = self.lock_cache.get(id)
                    && let Ok(parsed) = serde_json::from_str::<Value>(json_str)
                    && let Ok(res) = engine.evaluate_album_logic(&parsed)
                {
                    new_evaluated.insert(uid, res);
                }
            }
            self.evaluated_logic = new_evaluated;
            Ok(eval.manifest)
        })?;
        self.manifest = manifest;
        self.build_cache();
        Ok(())
    }

    pub fn clear(&mut self) {
        self.libraries_cache.clear();
        self.filters_cache.clear();
        self.facets_cache.clear();
        self.orders_cache.clear();
        self.shelves_cache.clear();
        self.uid_to_id.clear();
        self.id_to_uid.clear();
        self.next_uid = 1;
        self.dict.clear();
        self.track_lookup.clear();
        self.path_lookup.clear();
        self.evaluated_logic.clear();
        self.lock_cache.clear();
    }

    pub fn remove_album(&mut self, id: &str) {
        if let Some(uid) = self.id_to_uid.remove(id) {
            self.uid_to_id.remove(&uid);
            self.evaluated_logic.remove(&uid);
        }
        self.dict.remove(id);
        self.lock_cache.remove(id);
        self.path_lookup.retain(|_, v| v != id);
        self.track_lookup
            .retain(|_, v| v.get("albumId").and_then(|a| a.as_str()) != Some(id));
    }

    pub fn ingest(&mut self, id: &str, metadata_json: &str) -> Result<()> {
        let uid = self.next_uid;
        self.next_uid += 1;

        self.uid_to_id.insert(uid, id.to_string());
        self.id_to_uid.insert(id.to_string(), uid);
        self.lock_cache.insert(id.to_string(), metadata_json.to_string());

        let parsed: Value = serde_json::from_str(metadata_json)?;
        if let Some(album) = parsed.get("album")
            && let Some(info) = album.get("info")
        {
            if let Some(tracks) = parsed.get("tracks").and_then(Value::as_array) {
                for track in tracks {
                    if let Some(tinfo) = track.get("info") {
                        let rel = track
                            .get("file")
                            .and_then(|f| f.get("path"))
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        if !rel.is_empty() {
                            let tp_path = Path::new(id).join(rel);
                            let tp = tp_path.to_string_lossy().to_string();

                            let track_no =
                                track.get("tracknumber").cloned().unwrap_or_else(|| json!(0));
                            let disc_no =
                                track.get("discnumber").cloned().unwrap_or_else(|| json!(1));
                            let title =
                                track.get("title").cloned().unwrap_or_else(|| json!("Unknown"));
                            let artist = track
                                .get("artist")
                                .cloned()
                                .unwrap_or_else(|| json!("Unknown"));
                            let duration = tinfo
                                .get("duration_formatted")
                                .cloned()
                                .unwrap_or_else(|| json!("0:00"));
                            let duration_ms = tinfo
                                .get("duration_milliseconds")
                                .and_then(Value::as_u64)
                                .unwrap_or(0);

                            let track_light = json!({
                                "path": tp,
                                "trackNo": track_no,
                                "discNo": disc_no,
                                "title": title,
                                "artist": artist,
                                "duration": duration,
                                "durationMs": duration_ms,
                                "albumId": id
                            });
                            self.track_lookup.insert(tp.clone(), track_light);

                            let full_rel_path = Path::new(id).join(rel);
                            let normalized = full_rel_path
                                .to_string_lossy()
                                .trim_start_matches('/')
                                .to_string();
                            self.path_lookup.insert(normalized, id.to_string());
                        }
                    }
                }
            }

            let entry = json!({
                "id": id,
                "album": album.get("album"),
                "albumartist": album.get("albumartist"),
                "date": album.get("date"),
                "cover_path": album.get("covers").and_then(|c| c.get("main")).and_then(|m| m.get("file")).and_then(|f| f.get("path")),
                "cover_hash": album.get("covers").and_then(|c| c.get("main")).and_then(|m| m.get("file")).and_then(|f| f.get("address")),
                "duration_formatted": info.get("duration_formatted"),
                "total_discs": info.get("total_discs"),
                "total_tracks": info.get("total_tracks"),
                "virtual": info.get("virtual"),
                "keys": album.get("keys"),
                "colors": album.get("colors")
            });
            self.dict.insert(id.to_string(), entry);
        }

        let config_path = libvellum::lua::resolve_config_path().unwrap_or_default();
        let eval_res = get_or_init_lua_vm(&config_path, |engine| {
            engine.evaluate_album_logic(&parsed)
        })?;

        self.evaluated_logic.insert(uid, eval_res);
        Ok(())
    }

    #[must_use]
    pub fn get_album_json(&self, id: &str) -> Option<String> {
        self.lock_cache.get(id).cloned()
    }
}
