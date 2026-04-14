use crate::egui::theme;
use crate::server::library::models::TrackLock;
use eframe::egui;

pub fn render_tracklist(ui: &mut egui::Ui, tracks: &[TrackLock]) {
    let mut current_disc = 0;

    for track in tracks {
        if track.discnumber != current_disc {
            if current_disc != 0 { ui.add_space(12.0); ui.separator(); ui.add_space(12.0); }
            current_disc = track.discnumber;
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(13)))
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("Disc {}", current_disc)).size(12.0).color(egui::Color32::from_gray(102)));
                    });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 24.0), egui::Sense::click());
                    let bg = if resp.hovered() { egui::Color32::from_white_alpha(20) } else { egui::Color32::from_white_alpha(5) };
                    ui.painter().rect_filled(rect, 12.0, bg);
                    egui::Image::from_bytes("bytes://play_small.svg", include_bytes!("../../../../public/icons/20px/play_arrow.svg"))
                        .tint(egui::Color32::WHITE).paint_at(ui, rect.shrink(4.0));
                });
            });
            ui.add_space(10.0);
        }

        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
        if response.hovered() { ui.painter().rect_filled(rect, 10.0, egui::Color32::from_white_alpha(5)); }

        ui.painter().text(rect.left_center() + egui::vec2(10.0, 0.0), egui::Align2::LEFT_CENTER, track.tracknumber.to_string(), egui::FontId::monospace(13.0), egui::Color32::from_gray(136));
        ui.painter().text(rect.left_center() + egui::vec2(45.0, 0.0), egui::Align2::LEFT_CENTER, &track.title, egui::FontId::proportional(15.0), theme::TEXT_MAIN);
        ui.painter().text(rect.right_center() - egui::vec2(14.0, 0.0), egui::Align2::RIGHT_CENTER, &track.info.track_duration_time, egui::FontId::monospace(13.0), egui::Color32::from_gray(136));
    }
}
