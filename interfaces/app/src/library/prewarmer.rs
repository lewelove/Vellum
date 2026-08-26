use crate::api::fetch_cover_bytes;
use crate::library::collection::AlbumSummary;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};

pub struct Prewarmer {
    textures: HashMap<String, TextureHandle>,
    in_flight: HashSet<String>,
    loaded_rx: Receiver<(String, ColorImage)>,
    request_tx: Sender<(String, String, u32)>,
}

impl Prewarmer {
    #[must_use]
    pub fn new() -> Self {
        let (request_tx, request_rx) = std::sync::mpsc::channel::<(String, String, u32)>();
        let (loaded_tx, loaded_rx) = std::sync::mpsc::channel::<(String, ColorImage)>();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(8)
                .enable_all()
                .build()
                .expect("Failed to build background image loader runtime");

            rt.block_on(async move {
                while let Ok((hash, filter, size)) = request_rx.recv() {
                    let loaded_tx_clone = loaded_tx.clone();
                    tokio::spawn(async move {
                        if let Ok(bytes) = fetch_cover_bytes(&filter, size, &hash).await
                            && let Ok(dynamic_img) = image::load_from_memory(&bytes)
                        {
                            let rgb = dynamic_img.to_rgba8();
                            let (w, h) = rgb.dimensions();
                            let color_img = ColorImage::from_rgba_unmultiplied(
                                [w as usize, h as usize],
                                rgb.as_raw(),
                            );
                            let _ = loaded_tx_clone.send((hash, color_img));
                        }
                    });
                }
            });
        });

        Self {
            textures: HashMap::new(),
            in_flight: HashSet::new(),
            loaded_rx,
            request_tx,
        }
    }

    pub fn prewarm_all(&mut self, albums: &[AlbumSummary], filter: &str, size_px: u32) {
        for album in albums {
            if let Some(ref hash) = album.cover_hash {
                if !self.textures.contains_key(hash) && !self.in_flight.contains(hash) {
                    self.in_flight.insert(hash.clone());
                    let _ = self.request_tx.send((hash.clone(), filter.to_string(), size_px));
                }
            }
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        let mut loaded_any = false;
        while let Ok((hash, color_img)) = self.loaded_rx.try_recv() {
            self.in_flight.remove(&hash);
            let handle = ctx.load_texture(
                format!("cover-{hash}"),
                color_img,
                TextureOptions::LINEAR,
            );
            self.textures.insert(hash, handle);
            loaded_any = true;
        }
        if loaded_any {
            ctx.request_repaint();
        }
    }

    #[must_use]
    pub fn get(&self, hash: &str) -> Option<&TextureHandle> {
        self.textures.get(hash)
    }
}
