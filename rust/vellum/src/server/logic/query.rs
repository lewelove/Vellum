use super::LogicEngine;
use roaring::RoaringBitmap;
use serde_json::{Value, json};

impl LogicEngine {
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
