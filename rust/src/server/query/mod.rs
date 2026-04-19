use anyhow::Result;
use indexmap::IndexMap;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LogicManifest {
    pub groupers: IndexMap<String, GrouperDef>,
    pub sorters: IndexMap<String, SorterDef>,
    pub collections: IndexMap<String, CollectionDef>,
}

impl LogicManifest {
    pub fn normalize(&mut self) {
        let global_groupers: HashSet<String> = self.groupers.iter()
            .filter(|(_, g)| !g.strict)
            .map(|(id, _)| id.clone())
            .collect();

        let global_sorters: HashSet<String> = self.sorters.iter()
            .filter(|(_, s)| !s.strict)
            .map(|(id, _)| id.clone())
            .collect();

        for (_, collection) in self.collections.iter_mut() {
            let mut allowed_g_ids = HashSet::new();
            for g in &collection.groupers {
                allowed_g_ids.insert(g.clone());
            }
            if !collection.strict {
                for g in &global_groupers {
                    allowed_g_ids.insert(g.clone());
                }
            }

            let mut allowed_s_ids = HashSet::new();
            for s in &collection.sorters {
                allowed_s_ids.insert(s.clone());
            }
            if !collection.strict {
                for s in &global_sorters {
                    allowed_s_ids.insert(s.clone());
                }
            }

            collection.allowed_groupers = self.groupers.keys()
                .filter(|k| allowed_g_ids.contains(*k))
                .cloned()
                .collect();

            collection.allowed_sorters = self.sorters.keys()
                .filter(|k| allowed_s_ids.contains(*k))
                .cloned()
                .collect();
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GrouperDef {
    pub label: String,
    pub select: String,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SorterDef {
    pub label: String,
    pub order_by: String,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CollectionDef {
    pub label: String,
    pub filter: String,
    #[serde(default)]
    pub strict: bool,
    #[serde(default)]
    pub groupers: Vec<String>,
    #[serde(default)]
    pub sorters: Vec<String>,
    #[serde(skip_deserializing)]
    pub allowed_groupers: Vec<String>,
    #[serde(skip_deserializing)]
    pub allowed_sorters: Vec<String>,
}

pub struct QueryEngine {
    conn: Connection,
    pub manifest: LogicManifest,
    collections_cache: HashMap<String, HashSet<u32>>,
    facets_cache: HashMap<String, HashMap<String, HashSet<u32>>>,
    sorters_cache: HashMap<String, Vec<u32>>,
    uid_to_id: HashMap<u32, String>,
    pub dict: HashMap<String, Value>,
    pub track_lookup: HashMap<String, Value>,
    pub path_lookup: HashMap<String, String>,
}

const DEFAULT_LOGIC: &str = include_str!("logic.toml");

impl QueryEngine {
    pub fn new() -> Result<Self> {
        let logic_path = crate::expand_path("~/.config/vellum/logic.toml");
        if !logic_path.exists() {
            std::fs::write(&logic_path, DEFAULT_LOGIC)?;
        }
        
        let logic_content = std::fs::read_to_string(&logic_path)?;
        let mut manifest: LogicManifest = toml::from_str(&logic_content)?;
        manifest.normalize();

        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE albums (
                uid INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT UNIQUE,
                metadata TEXT
            )",[],
        )?;

        Ok(Self {
            conn,
            manifest,
            collections_cache: HashMap::new(),
            facets_cache: HashMap::new(),
            sorters_cache: HashMap::new(),
            uid_to_id: HashMap::new(),
            dict: HashMap::new(),
            track_lookup: HashMap::new(),
            path_lookup: HashMap::new(),
        })
    }

    pub fn reload_manifest(&mut self, path: &Path) -> Result<()> {
        let logic_content = std::fs::read_to_string(path)?;
        let mut manifest: LogicManifest = toml::from_str(&logic_content)?;
        manifest.normalize();
        self.manifest = manifest;
        self.build_cache()?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM albums",[])?;
        self.collections_cache.clear();
        self.facets_cache.clear();
        self.sorters_cache.clear();
        self.uid_to_id.clear();
        self.dict.clear();
        self.track_lookup.clear();
        self.path_lookup.clear();
        Ok(())
    }

    pub fn remove_album(&mut self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM albums WHERE id = ?1", [&id])?;
        self.dict.remove(id);
        Ok(())
    }

    pub fn ingest(&mut self, id: &str, metadata_json: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO albums (id, metadata) VALUES (?1, ?2)",[&id, metadata_json],
        )?;
        let uid = self.conn.last_insert_rowid() as u32;
        self.uid_to_id.insert(uid, id.to_string());

        if let Ok(parsed) = serde_json::from_str::<Value>(metadata_json) {
            if let Some(album) = parsed.get("album") {
                if let Some(info) = album.get("info") {
                    let mut tracks_light = Vec::new();
                    if let Some(tracks) = parsed.get("tracks").and_then(Value::as_array) {
                        for track in tracks {
                            if let Some(tinfo) = track.get("info") {
                                let tp = tinfo.get("track_library_path").and_then(Value::as_str).unwrap_or("").to_string();
                                let track_no = track.get("TRACKNUMBER").unwrap_or(&json!(0)).clone();
                                let disc_no = track.get("DISCNUMBER").unwrap_or(&json!(1)).clone();
                                let title = track.get("TITLE").unwrap_or(&json!("Unknown")).clone();
                                let artist = track.get("ARTIST").unwrap_or(&json!("Unknown")).clone();
                                let duration = tinfo.get("track_duration_time").unwrap_or(&json!("0:00")).clone();
                                
                                let track_light = json!({
                                    "path": tp,
                                    "trackNo": track_no,
                                    "discNo": disc_no,
                                    "title": title,
                                    "artist": artist,
                                    "duration": duration,
                                    "albumId": id
                                });
                                tracks_light.push(track_light.clone());
                                self.track_lookup.insert(tp.clone(), track_light);

                                let raw_rel = tinfo.get("track_path").and_then(Value::as_str).unwrap_or("");
                                let full_rel_path = Path::new(id).join(raw_rel);
                                let normalized = full_rel_path.to_string_lossy().trim_start_matches('/').to_string();
                                self.path_lookup.insert(normalized, id.to_string());
                            }
                        }
                    }

                    let entry = json!({
                        "id": id,
                        "ALBUM": album.get("ALBUM"),
                        "ALBUMARTIST": album.get("ALBUMARTIST"),
                        "DATE": album.get("DATE"),
                        "GENRE": album.get("GENRE"),
                        "cover_path": info.get("cover_path"),
                        "cover_hash": info.get("cover_hash"),
                        "album_duration_time": info.get("album_duration_time"),
                        "total_discs": info.get("total_discs"),
                        "total_tracks": info.get("total_tracks"),
                        "unix_added": info.get("unix_added"),
                        "tags": album.get("tags")
                    });
                    self.dict.insert(id.to_string(), entry);
                }
            }
        }

        Ok(())
    }

    pub fn build_cache(&mut self) -> Result<()> {
        self.collections_cache.clear();
        for (key, collection) in &self.manifest.collections {
            let sql = format!("SELECT uid FROM albums WHERE {}", collection.filter);
            let mut stmt = self.conn.prepare(&sql)?;
            let uids: HashSet<u32> = stmt.query_map([], |row| row.get(0))?.filter_map(Result::ok).collect();
            self.collections_cache.insert(key.clone(), uids);
        }

        self.sorters_cache.clear();
        for (key, sorter) in &self.manifest.sorters {
            let sql = format!("SELECT uid FROM albums ORDER BY {}", sorter.order_by);
            let mut stmt = self.conn.prepare(&sql)?;
            let uids: Vec<u32> = stmt.query_map([], |row| row.get(0))?.filter_map(Result::ok).collect();
            self.sorters_cache.insert(key.clone(), uids);
        }

        self.facets_cache.clear();
        for (key, grouper) in &self.manifest.groupers {
            let sql = format!("SELECT uid, {} FROM albums", grouper.select);
            let mut stmt = self.conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            
            let mut map: HashMap<String, HashSet<u32>> = HashMap::new();
            
            while let Some(row) = rows.next()? {
                let uid: u32 = row.get(0)?;
                let raw_val: rusqlite::types::Value = row.get(1)?;
                
                let val_str = match raw_val {
                    rusqlite::types::Value::Text(s) => s,
                    rusqlite::types::Value::Integer(i) => i.to_string(),
                    rusqlite::types::Value::Real(f) => f.to_string(),
                    _ => continue,
                };

                if let Ok(Value::Array(arr)) = serde_json::from_str(&val_str) {
                    for v in arr {
                        if let Some(s) = v.as_str() {
                            map.entry(s.trim().to_string()).or_default().insert(uid);
                        }
                    }
                } else if let Ok(Value::String(s)) = serde_json::from_str(&val_str) {
                    map.entry(s.trim().to_string()).or_default().insert(uid);
                } else {
                    map.entry(val_str.trim().to_string()).or_default().insert(uid);
                }
            }
            self.facets_cache.insert(key.clone(), map);
        }

        Ok(())
    }

    pub fn request_view(&self, collection: &str, sort: &str, filter_key: Option<&str>, filter_val: Option<&str>, reverse: bool) -> Vec<String> {
        let empty_set = HashSet::new();
        let collection_mask = self.collections_cache.get(collection).unwrap_or(&empty_set);
        let mut final_mask = collection_mask.clone();

        if let (Some(fk), Some(fv)) = (filter_key, filter_val) {
            if fk == "search" {
                let sql = "SELECT uid FROM albums WHERE json_extract(metadata, '$.album.ALBUM') LIKE ?1 OR json_extract(metadata, '$.album.ALBUMARTIST') LIKE ?1";
                if let Ok(mut stmt) = self.conn.prepare(sql) {
                    let pattern = format!("%{}%", fv);
                    if let Ok(match_uids_iter) = stmt.query_map([pattern], |row| row.get::<_, u32>(0)) {
                        let match_uids: HashSet<u32> = match_uids_iter.filter_map(Result::ok).collect();
                        final_mask.retain(|uid| match_uids.contains(uid));
                    }
                }
            } else if let Some(facet_vals) = self.facets_cache.get(fk) {
                if let Some(facet_mask) = facet_vals.get(fv) {
                    final_mask.retain(|uid| facet_mask.contains(uid));
                } else {
                    final_mask.clear();
                }
            }
        }

        let empty_vec = Vec::new();
        let sorted_uids = self.sorters_cache.get(sort).unwrap_or(&empty_vec);

        let mut res: Vec<String> = sorted_uids.iter()
            .filter(|uid| final_mask.contains(*uid))
            .filter_map(|uid| self.uid_to_id.get(uid).cloned())
            .collect();

        if reverse {
            res.reverse();
        }
        res
    }

    pub fn request_group(&self, collection: &str, grouper: &str) -> Vec<Value> {
        let empty_set = HashSet::new();
        let collection_mask = self.collections_cache.get(collection).unwrap_or(&empty_set);
        
        let mut results = Vec::new();
        if let Some(facet_map) = self.facets_cache.get(grouper) {
            for (val, mask) in facet_map {
                let count = mask.intersection(collection_mask).count();
                if count > 0 {
                    results.push(json!({
                        "value": val,
                        "label": val,
                        "count": count
                    }));
                }
            }
        }
        
        results.sort_by(|a, b| {
            let label_a = a.get("label").and_then(Value::as_str).unwrap_or("");
            let label_b = b.get("label").and_then(Value::as_str).unwrap_or("");
            alphanumeric_sort::compare_str(label_a, label_b)
        });

        results
    }

    pub fn get_album_json(&self, id: &str) -> Option<String> {
        let mut stmt = self.conn.prepare("SELECT metadata FROM albums WHERE id = ?1").ok()?;
        let mut rows = stmt.query([id]).ok()?;
        if let Some(row) = rows.next().ok()? {
            return row.get(0).ok();
        }
        None
    }
}
