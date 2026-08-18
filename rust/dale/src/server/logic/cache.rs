use super::{value_to_sort_key, LogicEngine, SortKey};
use roaring::RoaringBitmap;
use serde_json::{json, Value};

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
        self.groupers_cache.clear();
        self.precomputed_groups.clear();
        self.orders_cache.clear();
        self.shelves_cache.clear();

        self.populate_eval_caches();
        self.populate_groupers_cache();
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

    fn collect_raw_groupers(&mut self) {
        for (&uid, eval_res) in &self.evaluated_logic {
            if let Some(groupers) = eval_res.get("groupers").and_then(Value::as_object) {
                for (grouper_id, val) in groupers {
                    let grouper_entry = self.groupers_cache.entry(grouper_id.clone()).or_default();
                    if let Some(arr) = val.as_array() {
                        for item in arr {
                            let group_name = value_to_display_string(item);
                            if !group_name.is_empty() {
                                grouper_entry.entry(group_name).or_default().insert(uid);
                            }
                        }
                    } else {
                        let group_name = value_to_display_string(val);
                        if !group_name.is_empty() {
                            grouper_entry.entry(group_name).or_default().insert(uid);
                        }
                    }
                }
            }
        }
    }

    fn collect_library_views(&self) -> Vec<(String, Option<String>, RoaringBitmap)> {
        let mut views = Vec::new();
        let empty_bitmap = RoaringBitmap::new();

        if self.manifest.libraries.is_empty() {
            let mut all_uids = RoaringBitmap::new();
            for &uid in self.uid_to_id.keys() {
                all_uids.insert(uid);
            }
            views.push(("library".to_string(), None, all_uids));
        } else {
            for (lib_id, lib_def) in &self.manifest.libraries {
                let lib_mask = self.libraries_cache.get(lib_id).unwrap_or(&empty_bitmap);
                views.push((lib_id.clone(), None, lib_mask.clone()));

                let allowed_filters = if lib_def.allowed_filters.is_empty() {
                    lib_def.filters.clone()
                } else {
                    lib_def.allowed_filters.clone()
                };

                for filter_id in allowed_filters {
                    if let Some(f_mask) = self.filters_cache.get(&filter_id) {
                        let view_mask = lib_mask & f_mask;
                        views.push((lib_id.clone(), Some(filter_id), view_mask));
                    }
                }
            }
        }

        views
    }

    fn build_grouper_items(
        &self,
        lua_engine: Option<&libdale::lua::LuaEngine>,
        grouper_id: &str,
        view_mask: &RoaringBitmap,
    ) -> Vec<Value> {
        let grouper_def = self.manifest.groupers.get(grouper_id);
        let reverse = grouper_def.is_some_and(|g| g.reverse);

        let mut items: Vec<(SortKey, String, Value)> = Vec::new();

        if let Some(groups) = self.groupers_cache.get(grouper_id) {
            for (group_val, group_bitmap) in groups {
                let count = group_bitmap.intersection_len(view_mask) as usize;
                if count > 0 {
                    let (label, raw_sort) = lua_engine.map_or_else(
                        || (group_val.clone(), json!(group_val)),
                        |engine| {
                            engine.evaluate_grouper_format(
                                grouper_id,
                                group_val,
                                count as u64,
                            )
                        },
                    );

                    let sort_key = value_to_sort_key(&raw_sort);
                    let sort_str = sort_val_to_string(&raw_sort);

                    items.push((
                        sort_key,
                        group_val.clone(),
                        json!({
                            "value": group_val,
                            "label": label,
                            "sort": sort_str,
                            "count": count
                        }),
                    ));
                }
            }
        }

        items.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| alphanumeric_sort::compare_str(&a.1, &b.1))
        });

        if reverse {
            items.reverse();
        }

        items.into_iter().map(|(_, _, val)| val).collect()
    }

    fn precompute_views_for_target(
        &mut self,
        lua_engine: Option<&libdale::lua::LuaEngine>,
        lib_id: &str,
        filter_opt: Option<&str>,
        view_mask: &RoaringBitmap,
    ) {
        let allowed_groupers: Vec<String> = self.manifest.libraries.get(lib_id).map_or_else(
            || self.manifest.groupers.keys().cloned().collect(),
            |l| {
                if l.allowed_groupers.is_empty() {
                    if l.groupers.is_empty() {
                        self.manifest.groupers.keys().cloned().collect()
                    } else {
                        l.groupers.clone()
                    }
                } else {
                    l.allowed_groupers.clone()
                }
            },
        );

        for grouper_id in allowed_groupers {
            let final_list = self.build_grouper_items(lua_engine, &grouper_id, view_mask);
            self.precomputed_groups.insert(
                (lib_id.to_string(), filter_opt.map(ToString::to_string), grouper_id),
                final_list,
            );
        }
    }

    fn populate_groupers_cache(&mut self) {
        self.collect_raw_groupers();

        let lua_engine = libdale::lua::LuaEngine::new().ok();
        if let Some(ref engine) = lua_engine
            && self.config_path.exists()
        {
            let _ = engine.evaluate_config(&self.config_path);
        }

        let views = self.collect_library_views();

        for (lib_id, filter_opt, view_mask) in &views {
            self.precompute_views_for_target(
                lua_engine.as_ref(),
                lib_id,
                filter_opt.as_deref(),
                view_mask,
            );
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
