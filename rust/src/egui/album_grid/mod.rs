pub mod layout;
pub mod scroll;

use crate::egui::text;
use crate::egui::theme;
use crate::server::state::AppState;
use eframe::egui;
use layout::LayoutManager;
use rayon::prelude::*;
use scroll::ScrollEngine;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, mpsc};
use xxhash_rust::xxh64::xxh64;

pub struct GridController {
    pub layout: LayoutManager,
    pub scroll: ScrollEngine,
    pub text_cache: HashMap<u64, egui::TextureHandle>,
    pub cover_cache: HashMap<u64, egui::TextureHandle>,
    current_dpr: f32,
    font_db: cosmic_text::fontdb::Database,
    res_tx: mpsc::Sender<(u64, f32, tiny_skia::Pixmap, Option<egui::ColorImage>)>,
    res_rx: mpsc::Receiver<(u64, f32, tiny_skia::Pixmap, Option<egui::ColorImage>)>,
    pub queued_albums: HashSet<u64>,
}

impl GridController {
    pub fn new(damping: f32) -> Self {
        let mut db = cosmic_text::fontdb::Database::new();
        db.load_system_fonts();

        let (res_tx, res_rx) = mpsc::channel();

        Self {
            layout: LayoutManager::default(),
            scroll: ScrollEngine::new(damping * 66.0),
            text_cache: HashMap::new(),
            cover_cache: HashMap::new(),
            current_dpr: -1.0,
            font_db: db,
            res_tx,
            res_rx,
            queued_albums: HashSet::new(),
        }
    }

    fn add_analytic_aa_rect(
        &self,
        mesh: &mut egui::Mesh,
        rect: egui::Rect,
        color: egui::Color32,
        texture_id: egui::TextureId,
    ) {
        let i = mesh.vertices.len() as u32;
        let fringe = 0.5;

        let inner = rect.shrink(fringe);
        let outer = rect.expand(fringe);

        let size = rect.size();
        let uv_inset = fringe / size.x;
        let uv_inner = egui::Rect::from_min_max(
            egui::pos2(uv_inset, uv_inset),
            egui::pos2(1.0 - uv_inset, 1.0 - uv_inset),
        );
        let uv_outer = egui::Rect::from_min_max(
            egui::pos2(-uv_inset, -uv_inset),
            egui::pos2(1.0 + uv_inset, 1.0 + uv_inset),
        );

        let color_inner = color;
        let color_outer = color.linear_multiply(0.0);

        mesh.vertices.push(egui::epaint::Vertex {
            pos: inner.left_top(),
            uv: uv_inner.left_top(),
            color: color_inner,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: inner.right_top(),
            uv: uv_inner.right_top(),
            color: color_inner,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: inner.right_bottom(),
            uv: uv_inner.right_bottom(),
            color: color_inner,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: inner.left_bottom(),
            uv: uv_inner.left_bottom(),
            color: color_inner,
        });

        mesh.vertices.push(egui::epaint::Vertex {
            pos: outer.left_top(),
            uv: uv_outer.left_top(),
            color: color_outer,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: outer.right_top(),
            uv: uv_outer.right_top(),
            color: color_outer,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: outer.right_bottom(),
            uv: uv_outer.right_bottom(),
            color: color_outer,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: outer.left_bottom(),
            uv: uv_outer.left_bottom(),
            color: color_outer,
        });

        mesh.indices.extend_from_slice(&[
            i + 0, i + 1, i + 2, i + 0, i + 2, i + 3, i + 4, i + 5, i + 1, i + 4, i + 1, i + 0,
            i + 5, i + 6, i + 2, i + 5, i + 2, i + 1, i + 6, i + 7, i + 3, i + 6, i + 3, i + 2,
            i + 7, i + 4, i + 0, i + 7, i + 0, i + 3,
        ]);

        mesh.texture_id = texture_id;
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &Arc<AppState>) {
        let available = ui.available_size();
        self.layout.container_width = available.x;
        let dpr = ui.ctx().pixels_per_point();

        if (dpr - self.current_dpr).abs() > 0.001 {
            self.current_dpr = dpr;
            self.text_cache.clear();
            self.cover_cache.clear();
            self.queued_albums.clear();
            while self.res_rx.try_recv().is_ok() {}
        }

        let lib_guard = state.library.try_read();
        let albums = lib_guard.as_ref().map(|lib| lib.albums.as_slice()).unwrap_or(&[]);

        let total_cached = self.text_cache.len() + self.queued_albums.len();
        if total_cached < albums.len() {
            let mut missing = Vec::new();
            for a in albums {
                let id_hash = xxh64(a.id.as_bytes(), 0);
                if !self.text_cache.contains_key(&id_hash) && !self.queued_albums.contains(&id_hash) {
                    let mut thumb_path = None;
                    if !a.album_data.info.cover_hash.is_empty() {
                        if let Ok(config_guard) = state.config.try_read() {
                            if let Some(thumb_root) = &config_guard.thumbnail_root {
                                thumb_path = Some(
                                    thumb_root
                                        .join(format!("{}px", config_guard.thumbnail_size))
                                        .join(format!("{}.png", a.album_data.info.cover_hash)),
                                );
                            }
                        }
                    }

                    missing.push((
                        id_hash,
                        a.album_data.album.clone(),
                        a.album_data.albumartist.clone(),
                        thumb_path,
                    ));
                    self.queued_albums.insert(id_hash);
                }
            }

            if !missing.is_empty() {
                let res_tx = self.res_tx.clone();
                let db = self.font_db.clone();
                let card_size = self.layout.card_size;
                let text_blob_h =
                    self.layout.lh_title + self.layout.text_gap_lesser + self.layout.lh_artist;
                let task_dpr = self.current_dpr;
                let text_gamma = 1.2;
                let text_magic = 0.2;

                std::thread::spawn(move || {
                    missing
                        .into_par_iter()
                        .map_init(
                            || {
                                let font_system = cosmic_text::FontSystem::new_with_locale_and_db(
                                    "en-US".to_string(),
                                    db.clone(),
                                );
                                let context = swash::scale::ScaleContext::new();
                                (font_system, context)
                            },
                            |state_tuple, (id_hash, title, artist, thumb_path)| {
                                let (font_system, context) = state_tuple;
                                let pixmap = text::render_text_blob(
                                    &title,
                                    &artist,
                                    font_system,
                                    context,
                                    task_dpr,
                                    card_size,
                                    text_blob_h,
                                    text_gamma,
                                    text_magic,
                                );

                                let mut cover_opt = None;
                                if let Some(path) = thumb_path {
                                    if let Ok(img) = image::open(&path) {
                                        let rgba = img.into_rgba8();
                                        let size =[rgba.width() as usize, rgba.height() as usize];
                                        let pixels = rgba
                                            .pixels()
                                            .map(|p| {
                                                egui::Color32::from_rgba_unmultiplied(
                                                    p[0], p[1], p[2], p[3],
                                                )
                                            })
                                            .collect();
                                        cover_opt = Some(egui::ColorImage {
                                            size,
                                            pixels,
                                            source_size: egui::vec2(size[0] as f32, size[1] as f32),
                                        });
                                    }
                                }

                                let _ = res_tx.send((id_hash, task_dpr, pixmap, cover_opt));
                            },
                        )
                        .count();
                });
            }
        }

        let mut uploaded = 0;
        let mut repainted = false;
        while uploaded < 20 {
            if let Ok((id_hash, task_dpr, pixmap, cover_opt)) = self.res_rx.try_recv() {
                if (task_dpr - self.current_dpr).abs() < 0.001 {
                    let pixels: Vec<egui::Color32> = pixmap
                        .data()
                        .chunks_exact(4)
                        .map(|p| egui::Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
                        .collect();

                    let text_image = egui::ColorImage {
                        size: [pixmap.width() as usize, pixmap.height() as usize],
                        pixels,
                        source_size: egui::vec2(pixmap.width() as f32, pixmap.height() as f32),
                    };

                    let handle_text = ui.ctx().load_texture(
                        format!("text_blob_{id_hash}"),
                        text_image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.text_cache.insert(id_hash, handle_text);

                    if let Some(cover_image) = cover_opt {
                        let handle_cover = ui.ctx().load_texture(
                            format!("cover_blob_{id_hash}"),
                            cover_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.cover_cache.insert(id_hash, handle_cover);
                    }

                    repainted = true;
                }
                uploaded += 1;
            } else {
                break;
            }
        }

        if repainted {
            ui.ctx().request_repaint();
        }

        let row_count = (albums.len() as f32 / self.layout.cols() as f32).ceil() as usize;
        let viewport_height = available.y;
        let visible_rows = (viewport_height / self.layout.row_height()).ceil() as usize;
        let max_slots = (row_count.saturating_sub(visible_rows) as f32).max(0.0);

        let rect = ui.max_rect();
        if ui.rect_contains_pointer(rect) {
            let mut discrete_scroll = 0.0;
            ui.input(|i| {
                for e in &i.events {
                    if let egui::Event::MouseWheel { delta, .. } = e {
                        if delta.y > 0.1 {
                            discrete_scroll -= 1.0;
                        } else if delta.y < -0.1 {
                            discrete_scroll += 1.0;
                        }
                    }
                }
            });

            if discrete_scroll != 0.0 {
                self.scroll.scroll_discrete(discrete_scroll, max_slots);
            }
        }

        let dt = ui.input(|i| i.stable_dt);
        if self.scroll.update(self.layout.row_height(), dt) {
            ui.ctx().request_repaint();
        }

        let start_idx = (self.scroll.current_y / self.layout.row_height())
            .floor()
            .max(0.0) as usize;
        let end_idx =
            ((self.scroll.current_y + viewport_height) / self.layout.row_height()).ceil() as usize;
        let end_idx = end_idx.min(row_count.saturating_sub(1));

        let grid_width = (self.layout.cols() as f32 * self.layout.card_size)
            + (self.layout.cols().saturating_sub(1) as f32 * self.layout.gap_x);
        let offset_x = (available.x - grid_width).max(0.0) / 2.0;

        let text_blob_h =
            self.layout.lh_title + self.layout.text_gap_lesser + self.layout.lh_artist;

        for row_idx in start_idx..=end_idx {
            let row_y = self.layout.get_row_y(row_idx) - self.scroll.current_y;
            let start_item = row_idx * self.layout.cols();
            let end_item = (start_item + self.layout.cols()).min(albums.len());

            for (col_idx, item_idx) in (start_item..end_item).enumerate() {
                let col_x = offset_x + col_idx as f32 * (self.layout.card_size + self.layout.gap_x);
                let pos = rect.min + egui::vec2(col_x, row_y);

                let album = &albums[item_idx];
                let id_hash = xxh64(album.id.as_bytes(), 0);
                
                let cover_rect = egui::Rect::from_min_size(
                    pos,
                    egui::vec2(self.layout.card_size, self.layout.card_size),
                );

                let mut cover_tex_id = egui::TextureId::default();
                if let Some(handle) = self.cover_cache.get(&id_hash) {
                    cover_tex_id = handle.id();
                }

                if cover_tex_id == egui::TextureId::default() {
                    ui.painter()
                        .rect_filled(cover_rect, 0.0, theme::BG_DRAWER);
                } else {
                    let mut mesh = egui::Mesh::default();
                    self.add_analytic_aa_rect(
                        &mut mesh,
                        cover_rect,
                        egui::Color32::WHITE,
                        cover_tex_id,
                    );
                    ui.painter().add(egui::Shape::Mesh(mesh.into()));
                }

                let text_rect = egui::Rect::from_min_size(
                    pos + egui::vec2(0.0, self.layout.card_size + self.layout.text_gap_main),
                    egui::vec2(self.layout.card_size, text_blob_h),
                );

                if let Some(handle) = self.text_cache.get(&id_hash) {
                    let mut text_mesh = egui::Mesh::default();
                    self.add_analytic_aa_rect(
                        &mut text_mesh,
                        text_rect,
                        egui::Color32::WHITE,
                        handle.id(),
                    );
                    ui.painter().add(egui::Shape::Mesh(text_mesh.into()));
                }
            }
        }
    }
}
