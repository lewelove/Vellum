use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AlbumSummary {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover_hash: Option<String>,
    pub total_discs: Option<u32>,
    pub total_tracks: Option<u32>,
    pub duration_formatted: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GroupNode {
    pub value: String,
    pub label: String,
    pub sublabel: Option<String>,
    pub count: u64,
    pub children: Vec<GroupNode>,
}

#[derive(Default)]
pub struct CollectionStore {
    pub dict: HashMap<String, serde_json::Value>,
    pub library_view_ids: Vec<String>,
    pub sidebar_groups: HashMap<String, Vec<GroupNode>>,
    pub manifest: serde_json::Value,
}

impl CollectionStore {
    #[must_use]
    pub fn map_ids_to_albums(&self, ids: &[String]) -> Vec<AlbumSummary> {
        let mut albums = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(entry) = self.dict.get(id) {
                let title = entry
                    .get("album")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Untitled")
                    .to_string();
                let artist = entry
                    .get("albumartist")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Unknown")
                    .to_string();
                let cover_hash = entry
                    .get("cover_hash")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string);
                let total_discs = entry
                    .get("total_discs")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|v| u32::try_from(v).ok());
                let total_tracks = entry
                    .get("total_tracks")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|v| u32::try_from(v).ok());
                let duration_formatted = entry
                    .get("duration_formatted")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string);

                albums.push(AlbumSummary {
                    id: id.clone(),
                    title,
                    artist,
                    cover_hash,
                    total_discs,
                    total_tracks,
                    duration_formatted,
                });
            }
        }
        albums
    }

    #[must_use]
    pub fn get_libraries(&self) -> Vec<(String, String)> {
        let order = self
            .manifest
            .get("libraries_order")
            .and_then(serde_json::Value::as_array);

        let libs = self
            .manifest
            .get("libraries")
            .and_then(serde_json::Value::as_object);

        let Some(libs_map) = libs else {
            return Vec::new();
        };

        if let Some(order_arr) = order {
            return order_arr
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter_map(|key| {
                    let label = libs_map
                        .get(key)
                        .and_then(|v| v.get("label"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(key);
                    Some((key.to_string(), label.to_string()))
                })
                .collect();
        }

        libs_map
            .iter()
            .map(|(k, v)| {
                let label = v
                    .get("label")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(k);
                (k.clone(), label.to_string())
            })
            .collect()
    }

    #[must_use]
    pub fn get_groupers(&self) -> Vec<(String, String)> {
        let order = self
            .manifest
            .get("groupers_order")
            .and_then(serde_json::Value::as_array);

        let groupers = self
            .manifest
            .get("groupers")
            .and_then(serde_json::Value::as_object);

        let Some(groupers_map) = groupers else {
            return Vec::new();
        };

        if let Some(order_arr) = order {
            return order_arr
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter_map(|key| {
                    let label = groupers_map
                        .get(key)
                        .and_then(|v| v.get("label"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(key);
                    Some((key.to_string(), label.to_string()))
                })
                .collect();
        }

        groupers_map
            .iter()
            .map(|(k, v)| {
                let label = v
                    .get("label")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(k);
                (k.clone(), label.to_string())
            })
            .collect()
    }

    #[must_use]
    pub fn get_orders(&self) -> Vec<(String, String)> {
        let order = self
            .manifest
            .get("orders_order")
            .and_then(serde_json::Value::as_array);

        let orders = self
            .manifest
            .get("orders")
            .and_then(serde_json::Value::as_object);

        let Some(orders_map) = orders else {
            return Vec::new();
        };

        if let Some(order_arr) = order {
            return order_arr
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter_map(|key| {
                    let label = orders_map
                        .get(key)
                        .and_then(|v| v.get("label"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(key);
                    Some((key.to_string(), label.to_string()))
                })
                .collect();
        }

        orders_map
            .iter()
            .map(|(k, v)| {
                let label = v
                    .get("label")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(k);
                (k.clone(), label.to_string())
            })
            .collect()
    }
}
