pub mod tracks;

use crate::egui::theme;
use crate::server::state::AppState;
use crate::server::library::models::AlbumView;
use eframe::egui;
use std::sync::Arc;

pub fn render(ctx: &egui::Context, state: &Arc<AppState>, album: &AlbumView, on_close: impl FnOnce()) {
    let screen_rect = ctx.content_rect();
    let width = screen_rect.width() * 0.8;
    let height = screen_rect.height() * 0.85;
    
    egui::Area::new(egui::Id::new("modal_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            let response = ui.allocate_response(screen_rect.size(), egui::Sense::click());
            ui.painter().rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(150));
            if response.clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                on_close();
            }
        });

    egui::Window::new("modal_drawer")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(width, height))
        .title_bar(false)
        .frame(egui::Frame::NONE.fill(theme::BG_MAIN).corner_radius(16))
        .show(ctx, |ui| {
            let total_rect = ui.available_rect_before_wrap();
            let left_width = total_rect.width() * 0.45;
            
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                ui.allocate_ui_with_layout(egui::vec2(left_width, height), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    egui::Frame::NONE.fill(egui::Color32::from_rgb(31, 31, 31)).inner_margin(32.0).show(ui, |ui| {
                        let cover_size = left_width - 64.0;
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(cover_size, cover_size), egui::Sense::hover());
                        
                        let path = if let Ok(cfg) = state.config.try_read() {
                            cfg.library_root.join(&album.id).join(&album.album_data.info.cover_path)
                        } else { std::path::PathBuf::new() };

                        ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
                        let uri = format!("file://{}", path.to_string_lossy());
                        egui::Image::new(uri).corner_radius(0).paint_at(ui, rect);

                        ui.add_space(16.0);
                        ui.label(egui::RichText::new(&album.album_data.album).size(25.0).color(theme::TEXT_MAIN));
                        ui.label(egui::RichText::new(&album.album_data.albumartist).size(20.0).color(theme::TEXT_MUTED));
                        
                        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                            ui.add_space(32.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&album.album_data.info.album_duration_time).size(16.0).color(egui::Color32::from_gray(136)));
                                ui.label(egui::RichText::new("•").color(egui::Color32::from_gray(119)));
                                ui.label(egui::RichText::new(format!("{} Tracks", album.tracks.len())).size(16.0).color(egui::Color32::from_gray(136)));
                            });
                            if !album.album_data.date.is_empty() {
                                ui.label(egui::RichText::new(&album.album_data.date).size(16.0).color(egui::Color32::from_gray(136)));
                            }
                        });
                    });
                });

                ui.allocate_ui_with_layout(egui::vec2(total_rect.width() - left_width, height), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    egui::Frame::NONE.fill(theme::BG_DRAWER).inner_margin(32.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let mut btn = |ui: &mut egui::Ui, bytes: &'static [u8], uri: &'static str| {
                                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                                    let bg = if resp.hovered() { egui::Color32::from_white_alpha(20) } else { egui::Color32::from_white_alpha(5) };
                                    ui.painter().circle_filled(rect.center(), 18.0, bg);
                                    egui::Image::from_bytes(uri, bytes).tint(egui::Color32::WHITE).paint_at(ui, rect.shrink(8.0));
                                    resp
                                };

                                if btn(ui, include_bytes!("../../../../public/icons/24px/play_arrow.svg"), "bytes://play.svg").clicked() {}
                                ui.add_space(10.0);
                                if btn(ui, include_bytes!("../../../../public/icons/24px/code.svg"), "bytes://code.svg").clicked() {}
                                if btn(ui, include_bytes!("../../../../public/icons/24px/edit_document.svg"), "bytes://edit.svg").clicked() {}
                                if btn(ui, include_bytes!("../../../../public/icons/24px/folder.svg"), "bytes://folder.svg").clicked() {}
                                if btn(ui, include_bytes!("../../../../public/icons/24px/refresh.svg"), "bytes://refresh.svg").clicked() {}
                            });
                        });
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(12.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            tracks::render_tracklist(ui, &album.tracks);
                        });
                    });
                });
            });
        });
}
