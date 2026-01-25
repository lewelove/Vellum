use super::adapter::TrackData;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use lexical_sort::natural_lexical_cmp;

fn parse_sort_int(val: Option<&String>) -> i32 {
    match val {
        Some(s) => {
            let part = s.split('/').next().unwrap_or("0");
            part.trim().parse::<i32>().unwrap_or(0)
        },
        None => 0
    }
}

pub fn sort_tracks(tracks: &mut [TrackData]) {
    tracks.sort_by(|a, b| {
        let disc_a = parse_sort_int(a.get("DISCNUMBER"));
        let disc_b = parse_sort_int(b.get("DISCNUMBER"));
        if disc_a != disc_b {
            return disc_a.cmp(&disc_b);
        }

        let track_a = parse_sort_int(a.get("TRACKNUMBER"));
        let track_b = parse_sort_int(b.get("TRACKNUMBER"));
        if track_a != track_b {
            return track_a.cmp(&track_b);
        }

        natural_lexical_cmp(
            a.path.to_string_lossy().as_ref(),
            b.path.to_string_lossy().as_ref()
        )
    });
}

pub fn group_tracks(
    tracks: Vec<TrackData>, 
    keys: &[String]
) -> HashMap<Vec<String>, Vec<TrackData>> {
    let mut buckets: HashMap<Vec<String>, Vec<TrackData>> = HashMap::new();

    for track in tracks {
        let mut group_key = Vec::new();
        for k in keys {
            let val = track.get(k).cloned().unwrap_or_default();
            group_key.push(val);
        }
        buckets.entry(group_key).or_default().push(track);
    }

    buckets
}

pub fn resolve_anchor(tracks: &[TrackData], library_root: &Path) -> Option<PathBuf> {
    if tracks.is_empty() {
        return None;
    }

    let parents: Vec<PathBuf> = tracks.iter()
        .map(|t| t.path.parent().unwrap_or(&t.path).to_path_buf())
        .collect();

    let mut candidate = parents[0].clone();
    
    loop {
        if parents.iter().all(|p| p.starts_with(&candidate)) {
            break;
        }
        if !candidate.pop() {
            break;
        }
    }

    if !candidate.starts_with(library_root) {
        return None;
    }

    for p in &parents {
        match p.strip_prefix(&candidate) {
            Ok(rel) => {
                if rel.components().count() > 2 {
                    return None;
                }
            },
            Err(_) => return None,
        }
    }

    Some(candidate)
}
