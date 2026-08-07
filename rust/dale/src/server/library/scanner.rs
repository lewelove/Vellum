use libdale::models::LockFile;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub enum UpdateResult {
    Updated(String),
    Removed(String),
}

pub struct Library {
    pub root: PathBuf,
}

impl Library {
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn scan(&self, logic_engine: &mut crate::server::logic::LogicEngine) {
        log::info!("Scanning Library at {}", self.root.display());

        let lock_paths: Vec<PathBuf> = WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_name() == "album.lock.json")
            .map(|e| e.path().to_path_buf())
            .collect();

        let root = &self.root;
        let config_path = &logic_engine.config_path;

        let evaluated_items: Vec<_> = lock_paths
            .into_par_iter()
            .filter_map(|lock_path| {
                let content = std::fs::read_to_string(&lock_path).ok()?;
                let lock_data = serde_json::from_str::<LockFile>(&content).ok()?;
                let album_dir = lock_path.parent().unwrap_or(&lock_path);
                let expected_id = libdale::resolvers::rel_path(album_dir, root);
                let alb_id = if lock_data.album.id == expected_id {
                    lock_data.album.id
                } else {
                    expected_id
                };

                let eval_res = libdale::lua::get_or_init_lua_vm(config_path, |engine| {
                    let parsed: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or_default();
                    engine.evaluate_album_logic(&parsed)
                })
                .ok()?;

                Some((alb_id, content, eval_res))
            })
            .collect();

        logic_engine.clear();

        for (alb_id, content, eval_res) in evaluated_items {
            let _ = logic_engine.ingest_pre_evaluated(&alb_id, &content, eval_res);
        }

        logic_engine.build_cache();
        log::info!("Library Logic Engine Initialized.");
    }

    pub fn update_album(
        &self,
        folder_path_str: &str,
        logic_engine: &mut crate::server::logic::LogicEngine,
    ) -> UpdateResult {
        let folder_path = Path::new(folder_path_str);

        let abs_folder_path = if folder_path.is_absolute() {
            folder_path.to_path_buf()
        } else {
            self.root.join(folder_path)
        };

        let rel_path = abs_folder_path.strip_prefix(&self.root).unwrap_or(&abs_folder_path);
        let alb_id = rel_path.to_string_lossy().to_string();

        let lock_path = abs_folder_path.join("album.lock.json");
        if lock_path.exists()
            && let Ok(content) = std::fs::read_to_string(&lock_path)
            && let Ok(lock_data) = serde_json::from_str::<LockFile>(&content)
        {
            let parsed_alb_id = lock_data.album.id;
            logic_engine.remove_album(&parsed_alb_id);
            logic_engine.remove_album(&alb_id);
            let _ = logic_engine.ingest(&alb_id, &content);
            return UpdateResult::Updated(alb_id);
        }

        logic_engine.remove_album(&alb_id);
        UpdateResult::Removed(alb_id)
    }
}
