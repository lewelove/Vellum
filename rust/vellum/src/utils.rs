use std::path::PathBuf;

#[must_use]
pub fn expand_path(path_str: &str) -> PathBuf {
    if path_str.starts_with('~')
        && let Some(home) = dirs::home_dir() {
            if path_str == "~" {
                return home;
            }
            if let Some(stripped) = path_str.strip_prefix("~/") {
                return home.join(stripped);
            }
        }
    PathBuf::from(path_str)
}
