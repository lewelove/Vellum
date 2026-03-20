File: rust/src/compile/resolvers/native.rs
Role: Complex Native Algorithms

Description:
This file implements the advanced, high-performance logic for generating computed metadata. It includes algorithms for image analysis (K-Means color clustering, Chroma calculation), file path resolutions, and date cascading.

Imports:
`use crate::compile::builder::context::AlbumContext;`
- Provides access to the album state.
`use crate::compile::resolvers::standard;`
- Provides access to basic data retrieval functions.
`use image::GenericImageView;`
- Used for inspecting pixels of the cover image.
`use kmeans_colors::get_kmeans;`
- An external library utilized to cluster colors for palette extraction.
`use palette::{FromColor, Lab, Srgb};`
- Provides highly accurate color space conversions.
`use serde_json::{Value, json};`
- Used to output JSON.
`use std::collections::HashSet;`
- Used for deduplication.
`use std::path::Path;`
- For path comparisons.

Logic:
`pub fn resolve_date(ctx: &AlbumContext) -> String`
- Looks for multiple possible keys (`date`, `year`, `originalyear`) and returns the first one found to act as a unified release date.

`pub fn resolve_yyyy_mm(ctx: &AlbumContext, key: &str) -> String`
- Formats a date into a strict `YYYY-MM` representation. If only a year is known, it returns `YYYY-00`.

`pub fn resolve_genre(ctx: &AlbumContext) -> Vec<String>`
- Takes a loosely formatted genre input (which could be an array or a semicolon-separated string) and normalizes it into a deduplicated, clean list of strings.

`pub fn calculate_total_discs(tracks: &[Value]) -> u32`
- Scans all tracks, parses their disc numbers, adds them to a set to deduplicate them, and returns the total number of unique discs found.

`pub fn resolve_album_info_unix_added(ctx: &AlbumContext) -> u64`
- Defines an exact priority chain of historical "date added" metadata tags spanning various platforms (foobar, youtube, apple music) to accurately determine the absolute oldest timestamp the user added the album to their collection.

`pub fn resolve_custom_albumartist(ctx: &AlbumContext) -> String`
- Cascades through a few custom string variables to ensure a display artist is always present.

`pub fn rel_path(target: &Path, base: &Path) -> String`
- Takes an absolute path and a base path and strips the base path away, leaving a clean relative path string.

`pub fn resolve_cover_chroma(ctx: &AlbumContext) -> Option<Value>`
- Calculates how "colorful" or vibrant an album cover is.
- It iterates over every pixel in the cover image. Instead of standard HSV, it calculates color oppositions (Red-Green and Yellow-Blue channels). By measuring the standard deviation and mean of these opponent channels across the entire image, it calculates a score. A high score means the image has wildly varying and intense colors, while a score near 0 means it is entirely grayscale/monochrome.

`pub fn resolve_cover_entropy(ctx: &AlbumContext) -> Option<Value>`
- Measures the visual complexity of an image.
- It converts the image to grayscale and encodes it into a PNG buffer in memory. Since PNG compression algorithms struggle with high entropy (chaos/randomness) but excel at compressing flat solid colors, the resulting byte size of the PNG perfectly correlates to the visual complexity (entropy) of the image.

`pub fn resolve_cover_palette(ctx: &AlbumContext) -> Option<Value>`
- Extracts a visually striking color palette from the cover.
- It translates all pixels into the highly human-perceptual `LAB` color space. It uses the K-Means clustering algorithm to group the pixels into 16 dominant color clusters. It then merges similar colors that are too close in the LAB space to prevent duplicate shades. Finally, it scores the remaining colors based on how often they appear heavily multiplied by their vibrancy/chroma (ensuring an accent color wins over a boring gray background) and returns the top 8 colors as a formatted list of HEX strings with their ratios.

`pub fn resolve_comment(ctx: &AlbumContext) -> String`
- Generates a formatted descriptive string representing the album's release edition based on its release year, country, record label, and catalog number.

`pub fn resolve_lyrics_path(album_root: &Path, track_num: u32, disc_num: u32, total_discs: u32) -> Option<String>`
- Scans the `Lyrics` subfolder for `.lrc` or `.txt` files.
- It parses the filenames and attempts to match them to the current track and disc number. It prioritizes `.lrc` (synchronized lyrics) over plain text and returns the relative path to the file.
