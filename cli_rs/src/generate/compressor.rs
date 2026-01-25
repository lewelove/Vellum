use super::adapter::TrackData;
use crate::config::{LayoutConfig, LayoutItem};
use std::collections::{HashMap, HashSet};

pub fn get_layout_keys(layout: Option<&LayoutConfig>) -> HashSet<String> {
    let mut keys = HashSet::new();
    if let Some(cfg) = layout {
        for item in &cfg.layout {
            match item {
                LayoutItem::Key(s) => {
                    if s != "*" && s != "\n" && !s.starts_with("#") {
                        keys.insert(s.clone());
                    }
                },
                LayoutItem::Block(map) => {
                    for tags in map.values() {
                        for t in tags {
                            if t != "*" && t != "\n" {
                                keys.insert(t.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    keys
}

pub type AlbumPool = HashMap<String, String>;
pub type TrackPool = HashMap<String, String>;

pub fn compress(
    raw_tracks: &[TrackData],
    tracks_layout: Option<&LayoutConfig>,
) -> (AlbumPool, Vec<TrackPool>) {
    if raw_tracks.is_empty() {
        return (HashMap::new(), Vec::new());
    }

    let forced_track_keys = get_layout_keys(tracks_layout);

    let first_track = &raw_tracks[0].tags;
    let mut candidate_keys: HashSet<String> = first_track.keys().cloned().collect();

    for track in &raw_tracks[1..] {
        let keys: HashSet<String> = track.tags.keys().cloned().collect();
        candidate_keys.retain(|k| keys.contains(k));
    }

    let mut album_pool = HashMap::new();
    let mut keys_to_promote = Vec::new();

    for key in candidate_keys {
        let first_val = &first_track[&key];
        
        let is_identical = raw_tracks.iter().all(|t| &t.tags[&key] == first_val);

        if is_identical {
            if forced_track_keys.contains(&key) {
                continue;
            }

            keys_to_promote.push(key.clone());
            album_pool.insert(key, first_val.clone());
        }
    }

    let mut final_track_pools = Vec::new();
    
    for track in raw_tracks {
        let mut t_pool = track.tags.clone();
        
        for k in &keys_to_promote {
            t_pool.remove(k);
        }
        
        t_pool.remove("track_path_absolute");
        
        final_track_pools.push(t_pool);
    }

    (album_pool, final_track_pools)
}
