//! Root module for Lua unit tests.
//!
//! This module exports shared test fixture helpers and declares test submodules.

use std::fs;
use std::path::PathBuf;

mod fn_tests;
mod fs_tests;
mod serde_tests;

/// Temporary file structure for test operations.
///
/// You can use this structure to create temporary files that delete automatically when dropped.
pub struct TempFile(pub PathBuf);

impl TempFile {
    /// Creates a temporary file with the given content string.
    ///
    /// The file path uses process identity and nanosecond timestamps to ensure uniqueness.
    pub fn new(content: &str) -> Self {
        let mut path = std::env::temp_dir();
        let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let name = format!("dale_test_{}_{}.tmp", std::process::id(), nanos);
        path.push(name);
        fs::write(&path, content).expect("Failed to write temp test file");
        Self(path)
    }

    /// Returns the absolute file path as a string.
    pub fn path_str(&self) -> String {
        self.0.to_string_lossy().to_string()
    }
}

impl Drop for TempFile {
    /// Deletes the temporary file from disk.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
