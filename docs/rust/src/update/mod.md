File: rust/src/update/mod.rs
Role: Library State Synchronizer

Description:
This file ensures that the library state perfectly matches the actual physical files on disk. It intelligently determines which albums have been modified to orchestrate precise partial recompilations, keeping library updates blazing fast.

Imports:
`use rayon::prelude::*;`
- To distribute deep mathematical file verifications across all CPU cores.
`use sha2::{Digest, Sha256};`
- For creating mathematical signatures of folder states.
`use crate::compile; use crate::config::AppConfig;`
- Connecting to the compilation engine.

Logic:
`pub enum TrustState`
- Classifies the exact health condition of an album's data object (Valid, Missing completely, Broken Intent (user changed TOML), Broken Physics (user edited audio), Broken Assets (user changed cover)).

`pub async fn run(...) -> Result<()>`
- The top-level execution path for the update sequence.
- It builds a hash representing the library directory to ensure the user hasn't changed their config path. It finds every album folder and launches a high-speed parallel verification process across the entire library. It collects any albums flagged as "dirty" or modified and passes them as an explicit work queue to the compile engine. It also spins up a background task listening to the compiler; the exact second an album is successfully rebuilt, it hits the local server API to "hot reload" the specific album in the live memory.

`async fn validate_library_root(cache_dir: &Path, current_hash: &str) -> Result<()>`
- Checks a tiny state file to see if the configured library path has changed since the last run. If so, it wipes the cache and tells the server to initiate a full reset.

`fn verify_albums_parallel(...) -> Result<Vec<(PathBuf, u64, bool)>>`
- Distributes mathematical file verification across the CPU.
- For every album, it calculates a hyper-fast sum of the modification timestamps of the folder, the metadata, and the cover image. It compares this math to the internal cache. If the math matches perfectly, the album is skipped instantly. If the math differs, it drops into `verify_trust` for a granular scan. Returns a list of the modified albums needing attention.

`async fn handle_album_reload(...)`
- Submits an HTTP POST request to the local server asking it to reload a specific folder into RAM, seamlessly bypassing the need for a global server restart.

`fn get_mtime_sum(...) -> u64`
- Adds the filesystem modification timestamps of the root directory, the `metadata.toml` file, and any matching cover image.

`fn verify_trust(album_root: &Path) -> TrustState`
- The ultimate authority on album validity.
- It opens the compiled `metadata.lock.json`. It compares the internal record of file sizes and modified times against the actual physical files on the hard drive. If a user resized their album cover, or edited a song in Audacity, or tweaked a metadata string, the physical reality will diverge from the compiled record, and it will throw a broken state, demanding a recompilation.
