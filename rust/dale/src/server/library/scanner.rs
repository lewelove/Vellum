use libdale::models::LockFile;
use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

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

        let config_path = &logic_engine.config_path;

        let evaluated_items: Vec<_> = lock_paths
            .into_par_iter()
            .map_init(
                || {
                    let engine = libdale::lua::LuaEngine::new().ok()?;
                    engine.evaluate_config(config_path).ok()?;
                    Some(engine)
                },
                |engine_opt, lock_path| {
                    let engine = engine_opt.as_ref()?;
                    let content = std::fs::read_to_string(&lock_path).ok()?;
                    let lock_data = serde_json::from_str::<LockFile>(&content).ok()?;
                    let album_dir = lock_path.parent().unwrap_or(&lock_path).to_path_buf();
                    let alb_id = lock_data.album.id;

                    let parsed: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or_default();
                    let eval_res = engine.evaluate_album_logic(&parsed).ok()?;

                    Some((album_dir, alb_id, content, eval_res))
                },
            )
            .flatten()
            .collect();

        logic_engine.clear();

        for (album_dir, alb_id, content, eval_res) in evaluated_items {
            if let Err(e) = logic_engine.ingest_pre_evaluated(&album_dir, &alb_id, &content, eval_res, &self.root) {
                log::error!("Dedup validation error during startup scan: {e}");
            }
        }

        logic_engine.build_cache();
        log::info!("Library Logic Engine Initialized.");
    }
}
