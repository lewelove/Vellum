pub mod album_grid;
pub mod app;
pub mod navigation;
pub mod queue;
pub mod sidebar;
pub mod theme;
pub mod text;

use crate::server::state::AppState;
use anyhow::{anyhow, Result};
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

enum LoadState {
    Loading,
    Failed,
}

struct FileBytesLoader {
    requested: Arc<Mutex<HashMap<String, LoadState>>>,
}

impl FileBytesLoader {
    fn new() -> Self {
        Self {
            requested: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl egui::load::BytesLoader for FileBytesLoader {
    fn id(&self) -> &str {
        "vellum::FileBytesLoader"
    }

    fn load(&self, ctx: &egui::Context, uri: &str) -> Result<egui::load::BytesPoll, egui::load::LoadError> {
        let Some(path_str) = uri.strip_prefix("file://") else {
            return Err(egui::load::LoadError::NotSupported);
        };

        if let Some(bytes) = ctx.data(|d| d.get_temp::<Arc<[u8]>>(egui::Id::new(uri))) {
            return Ok(egui::load::BytesPoll::Ready {
                size: None,
                bytes: egui::load::Bytes::Shared(Arc::clone(&bytes)),
                mime: None,
            });
        }

        let mut requested = self.requested.lock().unwrap();
        if let Some(state) = requested.get(uri) {
            match state {
                LoadState::Loading => return Ok(egui::load::BytesPoll::Pending { size: None }),
                LoadState::Failed => {
                    return Err(egui::load::LoadError::NotSupported);
                }
            }
        }
        
        requested.insert(uri.to_owned(), LoadState::Loading);
        drop(requested);

        let uri_str = uri.to_owned();
        let path = std::path::PathBuf::from(path_str);
        let ctx_clone = ctx.clone();
        let requested_clone = Arc::clone(&self.requested);

        std::thread::spawn(move || {
            if let Ok(data) = std::fs::read(&path) {
                let slice: Arc<[u8]> = Arc::from(data.into_boxed_slice());
                ctx_clone.data_mut(|d| d.insert_temp(egui::Id::new(&uri_str), slice));
                ctx_clone.request_repaint();
            } else {
                let mut req = requested_clone.lock().unwrap();
                req.insert(uri_str, LoadState::Failed);
                ctx_clone.request_repaint(); 
            }
        });

        Ok(egui::load::BytesPoll::Pending { size: None })
    }

    fn forget(&self, uri: &str) {
        let mut requested = self.requested.lock().unwrap();
        requested.remove(uri);
    }

    fn forget_all(&self) {
        let mut requested = self.requested.lock().unwrap();
        requested.clear();
    }

    fn byte_size(&self) -> usize {
        0
    }
}

pub fn run(state: Arc<AppState>) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Vellum"),
        ..Default::default()
    };

    eframe::run_native(
        "Vellum",
        options,
        Box::new(|cc| {
            cc.egui_ctx.add_bytes_loader(Arc::new(FileBytesLoader::new()));
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app::VellumApp::new(state)))
        }),
    )
    .map_err(|e| anyhow!("eframe error: {:?}", e))
}
