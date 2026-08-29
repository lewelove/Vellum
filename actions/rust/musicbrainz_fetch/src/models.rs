use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetUrl {
    Release(String),
    ReleaseGroup(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwriteMode {
    Force,
    Preserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetDemand {
    Requested,
    Omitted,
}

impl From<bool> for TargetDemand {
    fn from(val: bool) -> Self {
        if val { Self::Requested } else { Self::Omitted }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetFlags {
    pub release: bool,
    pub release_group: bool,
    pub all_releases: bool,
}

pub struct ExecutionPlan {
    pub target: TargetUrl,
    pub output_dir: PathBuf,
    pub overwrite: OverwriteMode,
    pub flags: TargetFlags,
}

#[must_use]
pub fn parse_musicbrainz_url(input: &str) -> Option<TargetUrl> {
    let trimmed = input.trim();
    if let Some(id_str) = trimmed.split("musicbrainz.org/release-group/").nth(1) {
        let id = extract_mbid(id_str)?;
        return Some(TargetUrl::ReleaseGroup(id));
    }
    if let Some(id_str) = trimmed.split("musicbrainz.org/release/").nth(1) {
        let id = extract_mbid(id_str)?;
        return Some(TargetUrl::Release(id));
    }
    None
}

fn extract_mbid(raw: &str) -> Option<String> {
    let clean = raw
        .split('/')
        .next()
        .unwrap_or(raw)
        .split('?')
        .next()
        .unwrap_or(raw)
        .split('#')
        .next()
        .unwrap_or(raw)
        .trim();
    if clean.is_empty() {
        None
    } else {
        Some(clean.to_string())
    }
}
