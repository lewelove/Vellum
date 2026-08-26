use super::collection::{CollectionStore, GroupNode};
use super::sync::{OutboundMessage, SyncEngine};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HomeSubView {
    #[default]
    Libraries,
    Cabinets,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollReset {
    Reset,
    Preserve,
}

pub struct ViewState {
    pub home_sub_view: HomeSubView,
    pub active_library: String,
    pub active_library_filter: Option<String>,
    pub active_sort_key: Option<String>,
    pub is_reverse: bool,
    pub active_sidebar_grouper: Option<String>,
    pub active_filter_key: Option<String>,
    pub active_filter_val: Option<String>,
    pub focused_album_id: Option<String>,
    pub sidebar_width: f32,
    pub version: u64,
    pub reset_version: u64,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            home_sub_view: HomeSubView::Libraries,
            active_library: "all_albums".to_string(),
            active_library_filter: None,
            active_sort_key: Some("default".to_string()),
            is_reverse: false,
            active_sidebar_grouper: Some("genre".to_string()),
            active_filter_key: None,
            active_filter_val: None,
            focused_album_id: None,
            sidebar_width: 280.0,
            version: 0,
            reset_version: 0,
        }
    }
}

fn parse_group_nodes(raw_items: &[serde_json::Value]) -> Vec<GroupNode> {
    raw_items
        .iter()
        .map(|item| {
            let value = item
                .get("value")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            let label = item
                .get("label")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&value)
                .to_string();
            let sublabel = item
                .get("sublabel")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string);
            let count = item
                .get("count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            let children = item
                .get("children")
                .and_then(serde_json::Value::as_array)
                .map_or_else(Vec::new, |arr| parse_group_nodes(arr));

            GroupNode {
                value,
                label,
                sublabel,
                count,
                children,
            }
        })
        .collect()
}

impl ViewState {
    pub fn refresh_view(&mut self, sync: &SyncEngine, reset: ScrollReset) {
        if reset == ScrollReset::Reset {
            self.reset_version = self.reset_version.wrapping_add(1);
        }
        sync.send(OutboundMessage::ViewRequest {
            library: self.active_library.clone(),
            library_filter: self.active_library_filter.clone(),
            sort: self.active_sort_key.clone(),
            reverse: self.is_reverse,
            filter_key: self.active_filter_key.clone(),
            filter_val: self.active_filter_val.clone(),
        });
    }

    pub fn refresh_sidebar(&self, sync: &SyncEngine) {
        if let Some(ref grouper) = self.active_sidebar_grouper {
            sync.send(OutboundMessage::GroupRequest {
                library: self.active_library.clone(),
                library_filter: self.active_library_filter.clone(),
                key: grouper.clone(),
            });
        }
    }

    pub fn apply_filter(&mut self, key: String, val: String, sync: &SyncEngine) {
        if self.active_filter_key.as_deref() == Some(&key)
            && self.active_filter_val.as_deref() == Some(&val)
        {
            self.active_filter_key = None;
            self.active_filter_val = None;
        } else {
            self.active_filter_key = Some(key);
            self.active_filter_val = Some(val);
        }
        self.refresh_view(sync, ScrollReset::Reset);
    }

    pub fn handle_inbound_message(
        &mut self,
        json_val: &serde_json::Value,
        collection: &mut CollectionStore,
        sync: &SyncEngine,
    ) {
        let Some(msg_type) = json_val.get("type").and_then(serde_json::Value::as_str) else {
            return;
        };

        match msg_type {
            "INIT_DICT" => {
                if let Some(dict) = json_val.get("dict").and_then(serde_json::Value::as_object) {
                    collection.dict = dict
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                }
                if let Some(manifest) = json_val.get("manifest") {
                    collection.manifest = manifest.clone();
                    if let Some(libs_order) =
                        manifest.get("libraries_order").and_then(serde_json::Value::as_array)
                        && let Some(first) = libs_order.first().and_then(serde_json::Value::as_str)
                    {
                        self.active_library = first.to_string();
                    }
                    if let Some(groupers_order) =
                        manifest.get("groupers_order").and_then(serde_json::Value::as_array)
                        && let Some(first_g) = groupers_order.first().and_then(serde_json::Value::as_str)
                    {
                        self.active_sidebar_grouper = Some(first_g.to_string());
                    }
                }
                self.refresh_view(sync, ScrollReset::Reset);
                self.refresh_sidebar(sync);
            }
            "VIEW_DATA" => {
                if let Some(ids) = json_val.get("ids").and_then(serde_json::Value::as_array) {
                    collection.library_view_ids = ids
                        .iter()
                        .filter_map(|v| v.as_str().map(ToString::to_string))
                        .collect();
                    self.version = self.version.wrapping_add(1);
                }
            }
            "GROUP_RESULT" => {
                if let (Some(key), Some(result_arr)) = (
                    json_val.get("key").and_then(serde_json::Value::as_str),
                    json_val.get("result").and_then(serde_json::Value::as_array),
                ) {
                    let nodes = parse_group_nodes(result_arr);
                    collection.sidebar_groups.insert(key.to_string(), nodes);
                }
            }
            "ALBUMS_UPDATED" => {
                if let Some(updated) = json_val.get("updated").and_then(serde_json::Value::as_object) {
                    for (id, val) in updated {
                        collection.dict.insert(id.clone(), val.clone());
                    }
                }
                if let Some(removed) = json_val.get("removed").and_then(serde_json::Value::as_array) {
                    for id_val in removed {
                        if let Some(id) = id_val.as_str() {
                            collection.dict.remove(id);
                        }
                    }
                }
                self.refresh_view(sync, ScrollReset::Preserve);
                self.refresh_sidebar(sync);
            }
            _ => {}
        }
    }
}
