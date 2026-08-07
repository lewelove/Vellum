use super::{LogicEngine, SortKey, value_to_sort_key};
use roaring::RoaringBitmap;
use serde_json::{Value, json};

fn value_to_display_string(val: &Value) -> String {
    val.as_str().map_or_else(|| val.to_string(), ToString::to_string)
}

fn sort_val_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Array(arr) => arr.iter().map(sort_val_to_string).collect::<Vec<_>>().join(" "),
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
            entries.into_iter().map(|(_, v)| sort_val_to_string(v)).collect::<Vec<_>>().join(" ")
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
    }
}

impl LogicEngine {
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

                    let mut process_item = |obj: &Value| {
                        let (display_val, sort_val) = obj.as_object().map_or_else(
                            || {
                                let s = value_to_display_string(obj);
                                (s.clone(), Value::String(s))
                            },
                            |map| {
                                let v = map.get("value").map_or_else(
                                    String::new,
                                    value_to_display_string,
                                );
                                let s = map.get("sort").unwrap_or(&Value::Null);
                                let s_val = if s.is_null() {
                                    Value::String(v.clone())
                                } else {
                                    s.clone()
                                };
                                (v, s_val)
                            },
                        );

                        let sort_key = value_to_sort_key(&sort_val);

                        facet_map
                            .entry(display_val)
                            .and_modify(|(existing_key, existing_str, bitmap)| {
                                if sort_key < *existing_key {
                                    *existing_key = sort_key.clone();
                                    *existing_str = sort_val_to_string(&sort_val);
                                }
                                bitmap.insert(uid);
                            })
                            .or_insert_with(|| {
                                let mut bm = RoaringBitmap::new();
                                bm.insert(uid);
                                (sort_key, sort_val_to_string(&sort_val), bm)
                            });
                    };

                    if let Some(arr) = val.as_array() {
                        for item in arr {
                            process_item(item);
                        }
                    } else {
                        process_item(val);
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
            if let Some(shelf_uids) = self.shelves_cache.get_mut(shelf_key) {
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
}
