use anyhow::Result;
use libvellum::lua::{get_or_init_lua_vm, reset_lua_vm, LogicManifest};
use roaring::RoaringBitmap;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum SortKey {
    Number(i64),
    Float(f64),
    String(String),
    Tuple(Vec<Self>),
}

impl Eq for SortKey {}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.cmp(b),
            (Self::Float(a), Self::Float(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Self::Number(a), Self::Float(b)) => {
                let a_f = a.to_string().parse::<f64>().unwrap_or(0.0);
                a_f.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Self::Float(a), Self::Number(b)) => {
                let b_f = b.to_string().parse::<f64>().unwrap_or(0.0);
                a.partial_cmp(&b_f).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Self::String(a), Self::String(b)) => alphanumeric_sort::compare_str(a, b),
            (Self::Tuple(a), Self::Tuple(b)) => {
                for (x, y) in a.iter().zip(b.iter()) {
                    let res = x.cmp(y);
                    if res != std::cmp::Ordering::Equal {
                        return res;
                    }
                }
                a.len().cmp(&b.len())
            }
            (Self::Number(_) | Self::Float(_), _) | (Self::String(_), Self::Tuple(_)) => {
                std::cmp::Ordering::Less
            }
            (_, Self::Number(_) | Self::Float(_)) | (Self::Tuple(_), Self::String(_)) => {
                std::cmp::Ordering::Greater
            }
        }
    }
}

pub fn value_to_sort_key(val: &Value) -> SortKey {
    match val {
        Value::Number(n) => n.as_i64().map_or_else(
            || n.as_f64().map_or(SortKey::Number(0), SortKey::Float),
            SortKey::Number,
        ),
        Value::String(s) => SortKey::String(s.clone()),
        Value::Array(arr) => SortKey::Tuple(arr.iter().map(value_to_sort_key).collect()),
        Value::Object(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|(k1, _), (k2, _)| {
                let n1 = k1.parse::<usize>().ok();
                let n2 = k2.parse::<usize>().ok();
                match (n1, n2) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    _ => k1.cmp(k2),
                }
            });
            SortKey::Tuple(entries.into_iter().map(|(_, v)| value_to_sort_key(v)).collect())
        }
        Value::Bool(b) => SortKey::Number(i64::from(*b)),
        Value::Null => SortKey::String(String::new()),
    }
}

pub struct LogicEngine {
    pub manifest: LogicManifest,
    libraries_cache: HashMap<String, RoaringBitmap>,
    filters_cache: HashMap<String, RoaringBitmap>,
    facets_cache: HashMap<String, HashMap<String, RoaringBitmap>>,
    orders_cache: HashMap<String, Vec<u32>>,
    shelves_cache: HashMap<String, Vec<u32>>,
    uid_to_id: HashMap<u32, String>,
    id_to_uid: HashMap<String, u32>,
    next_uid: u32,
    pub dict: HashMap<String, Value>,
    pub track_lookup: HashMap<String, Value>,
    pub path_lookup: HashMap<String, String>,
    evaluated_logic: HashMap<u32, Value>,
    lock_cache: HashMap<String, String>,
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

    pub fn build_cache(&mut self) {
        self.libraries_cache.clear();
        self.filters_cache.clear();
        self.facets_cache.clear();
        self.orders_cache.clear();
        self.shelves_cache.clear();

        self.populate_eval_caches();
        self.populate_shelves_cache();
        self.populate_orders_cache();
    }

    fn populate_eval_caches(&mut self) {
        for (&uid, eval_res) in &self.evaluated_logic {
            if let Some(libs) = eval_res.get("libraries").and_then(Value::as_object) {
                for (lib_id, is_match) in libs {
                    if is_match.as_bool().unwrap_or(false) {
                        self.libraries_cache
                            .entry(lib_id.clone())
                            .or_default()
                            .insert(uid);
                    }
                }
            }

            if let Some(filters) = eval_res.get("filters").and_then(Value::as_object) {
                for (filter_id, is_match) in filters {
                    if is_match.as_bool().unwrap_or(false) {
                        self.filters_cache
                            .entry(filter_id.clone())
                            .or_default()
                            .insert(uid);
                    }
                }
            }

            if let Some(groupers) = eval_res.get("groupers").and_then(Value::as_object) {
                for (grouper_id, val) in groupers {
                    let facet_map = self.facets_cache.entry(grouper_id.clone()).or_default();
                    if let Some(arr) = val.as_array() {
                        for item in arr {
                            let s = item.as_str().map_or_else(|| item.to_string(), ToString::to_string);
                            facet_map.entry(s).or_default().insert(uid);
                        }
                    } else if let Some(s) = val.as_str() {
                        facet_map.entry(s.to_string()).or_default().insert(uid);
                    } else {
                        facet_map.entry(val.to_string()).or_default().insert(uid);
                    }
                }
            }

            if let Some(shelves) = eval_res.get("shelves").and_then(Value::as_object) {
                for (shelf_id, is_match) in shelves {
                    if is_match.as_bool().unwrap_or(false) {
                        self.shelves_cache
                            .entry(shelf_id.clone())
                            .or_default()
                            .push(uid);
                    }
                }
            }
        }
    }

    fn populate_shelves_cache(&mut self) {
        for (shelf_key, shelf) in &self.manifest.shelves {
            if let Some(file_path) = &shelf.file {
                let expanded = libvellum::utils::expand_path(file_path);
                if let Ok(content) = std::fs::read_to_string(&expanded) {
                    let lines: Vec<u32> = content
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty() && !l.starts_with('#'))
                        .filter_map(|l| self.id_to_uid.get(&l).copied())
                        .collect();
                    self.shelves_cache.insert(shelf_key.clone(), lines);
                }
            } else if let Some(shelf_uids) = self.shelves_cache.get_mut(shelf_key) {
                let default_key = json!("");
                let mut pairs: Vec<(u32, SortKey)> = shelf_uids
                    .iter()
                    .map(|&uid| {
                        let raw_key = self
                            .evaluated_logic
                            .get(&uid)
                            .and_then(|eval| eval.get("shelf_sorts"))
                            .and_then(|sorts| sorts.get(shelf_key))
                            .unwrap_or(&default_key);
                        (uid, value_to_sort_key(raw_key))
                    })
                    .collect();

                pairs.sort_by(|a, b| (&a.1, a.0).cmp(&(&b.1, b.0)));
                if shelf.reverse {
                    pairs.reverse();
                }

                *shelf_uids = pairs.into_iter().map(|(uid, _)| uid).collect();
            }
        }
    }

    fn populate_orders_cache(&mut self) {
        for order_id in self.manifest.orders.keys() {
            let mut order_pairs: Vec<(u32, SortKey)> = Vec::new();
            for (&uid, eval_res) in &self.evaluated_logic {
                let default_key = json!("");
                let raw_key = eval_res
                    .get("orders")
                    .and_then(|o| o.get(order_id))
                    .unwrap_or(&default_key);
                let sort_key = value_to_sort_key(raw_key);
                order_pairs.push((uid, sort_key));
            }

            order_pairs.sort_by(|a, b| (&a.1, a.0).cmp(&(&b.1, b.0)));
            let is_reverse = self
                .manifest
                .orders
                .get(order_id)
                .is_some_and(|o| o.reverse);
            if is_reverse {
                order_pairs.reverse();
            }

            let sorted_uids: Vec<u32> = order_pairs.into_iter().map(|(uid, _)| uid).collect();
            self.orders_cache.insert(order_id.clone(), sorted_uids);
        }
    }

    pub fn request_view(
        &self,
        library: &str,
        library_filter: Option<&str>,
        sort: &str,
        filter_key: Option<&str>,
        filter_val: Option<&str>,
        reverse: bool,
    ) -> Vec<String> {
        let empty_bitmap = RoaringBitmap::new();
        let library_mask = self.libraries_cache.get(library).unwrap_or(&empty_bitmap);
        let mut final_mask = library_mask.clone();

        if let Some(lf) = library_filter
            && let Some(f_mask) = self.filters_cache.get(lf)
        {
            final_mask &= f_mask;
        }

        if let (Some(fk), Some(fv)) = (filter_key, filter_val) {
            if fk == "search" {
                let needle = fv.to_lowercase();
                let mut searched = RoaringBitmap::new();
                for uid in &final_mask {
                    if let Some(id) = self.uid_to_id.get(&uid)
                        && let Some(album) = self.dict.get(id)
                    {
                        let title = album
                            .get("album")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_lowercase();
                        let artist = album
                            .get("albumartist")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_lowercase();
                        if title.contains(&needle) || artist.contains(&needle) {
                            searched.insert(uid);
                        }
                    }
                }
                final_mask = searched;
            } else if let Some(facet_vals) = self.facets_cache.get(fk) {
                if let Some(facet_mask) = facet_vals.get(fv) {
                    final_mask &= facet_mask;
                } else {
                    final_mask.clear();
                }
            }
        }

        let empty_vec = Vec::new();
        let sorted_uids = self.orders_cache.get(sort).unwrap_or(&empty_vec);

        let mut res: Vec<String> = sorted_uids
            .iter()
            .filter(|uid| final_mask.contains(**uid))
            .filter_map(|uid| self.uid_to_id.get(uid).cloned())
            .collect();

        if reverse {
            res.reverse();
        }
        res
    }

    pub fn request_shelf_view(&self, shelf_key: &str) -> Vec<String> {
        let empty_vec = Vec::new();
        let uids = self.shelves_cache.get(shelf_key).unwrap_or(&empty_vec);
        uids.iter()
            .filter_map(|uid| self.uid_to_id.get(uid).cloned())
            .collect()
    }

    pub fn request_group(
        &self,
        library: &str,
        library_filter: Option<&str>,
        grouper: &str,
    ) -> Vec<Value> {
        let empty_bitmap = RoaringBitmap::new();
        let library_mask = self.libraries_cache.get(library).unwrap_or(&empty_bitmap);
        let mut final_mask = library_mask.clone();

        if let Some(lf) = library_filter
            && let Some(f_mask) = self.filters_cache.get(lf)
        {
            final_mask &= f_mask;
        }

        let mut results = Vec::new();
        if let Some(facet_map) = self.facets_cache.get(grouper) {
            for (val, mask) in facet_map {
                let count = mask.intersection_len(&final_mask) as usize;
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
            let label_a = a
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_lowercase();
            let label_b = b
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_lowercase();
            alphanumeric_sort::compare_str(&label_a, &label_b)
        });

        results
    }

    #[must_use]
    pub fn get_album_json(&self, id: &str) -> Option<String> {
        self.lock_cache.get(id).cloned()
    }

    pub fn find_ids(&self, query_str: &str) -> Vec<String> {
        let q = query_str.trim();
        if q.is_empty() {
            return self.uid_to_id.values().cloned().collect();
        }

        if let Some(uids) = self.libraries_cache.get(q) {
            return uids
                .iter()
                .filter_map(|uid| self.uid_to_id.get(&uid).cloned())
                .collect();
        }
        if let Some(uids) = self.filters_cache.get(q) {
            return uids
                .iter()
                .filter_map(|uid| self.uid_to_id.get(&uid).cloned())
                .collect();
        }
        if let Some(uids) = self.shelves_cache.get(q) {
            return uids
                .iter()
                .filter_map(|uid| self.uid_to_id.get(uid).cloned())
                .collect();
        }

        let needle = q.to_lowercase();
        self.uid_to_id
            .values()
            .filter_map(|id| {
                if let Some(album) = self.dict.get(id) {
                    let title = album
                        .get("album")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_lowercase();
                    let artist = album
                        .get("albumartist")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_lowercase();
                    if id.to_lowercase().contains(&needle)
                        || title.contains(&needle)
                        || artist.contains(&needle)
                    {
                        return Some(id.clone());
                    }
                }
                None
            })
            .collect()
    }
}
