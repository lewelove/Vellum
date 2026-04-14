use crate::egui::theme;
use crate::egui::logic::groupers;
use crate::server::state::AppState;
use eframe::egui;
use eframe::egui::Widget;
use std::sync::Arc;

pub struct SidebarController {
    pub active_shelf: String,
    pub group_key: String,
    pub sort_key: String,
    pub sort_reverse: bool,
    pub filter_key: Option<String>,
    pub filter_val: Option<String>,
}

impl Default for SidebarController {
    fn default() -> Self {
        Self {
            active_shelf: "library".to_string(),
            group_key: "genre".to_string(),
            sort_key: "default".to_string(),
            sort_reverse: false,
            filter_key: None,
            filter_val: None,
        }
    }
}

impl SidebarController {
    fn combo_style(&self, ui: &mut egui::Ui) {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.bg_fill = egui::Color32::from_white_alpha(3);
        widgets.hovered.bg_fill = egui::Color32::from_white_alpha(13);
        widgets.active.bg_fill = egui::Color32::from_white_alpha(13);
        widgets.inactive.corner_radius = egui::CornerRadius::same(10);
        widgets.hovered.corner_radius = egui::CornerRadius::same(10);
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &Arc<AppState>) {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
        ui.add_space(12.0);

        ui.vertical(|ui| {
            self.combo_style(ui);
            
            ui.horizontal(|ui| {
                egui::Image::from_bytes("bytes://shelf.svg", include_bytes!("../../../public/icons/outlined/20px/auto_stories.svg")).tint(theme::TEXT_MUTED).ui(ui);
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("shelf_menu").width(ui.available_width()).selected_text(&self.active_shelf).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.active_shelf, "library".to_string(), "Entire Library");
                });
            });

            ui.horizontal(|ui| {
                egui::Image::from_bytes("bytes://group.svg", include_bytes!("../../../public/icons/outlined/20px/stack_group.svg")).tint(theme::TEXT_MUTED).ui(ui);
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("group_menu").width(ui.available_width()).selected_text(&self.group_key).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.group_key, "genre".to_string(), "Genre");
                    ui.selectable_value(&mut self.group_key, "decade".to_string(), "Decade");
                    ui.selectable_value(&mut self.group_key, "year_added".to_string(), "Year Added");
                    ui.selectable_value(&mut self.group_key, "chroma".to_string(), "Chroma");
                });
            });

            ui.horizontal(|ui| {
                egui::Image::from_bytes("bytes://sort.svg", include_bytes!("../../../public/icons/outlined/20px/swap_vert.svg")).tint(theme::TEXT_MUTED).ui(ui);
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("sort_menu").width(ui.available_width() - 44.0).selected_text(&self.sort_key).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_key, "default".to_string(), "Default");
                    ui.selectable_value(&mut self.sort_key, "date_added".to_string(), "Date Added");
                });

                let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                let bg = if resp.hovered() { egui::Color32::from_white_alpha(13) } else { egui::Color32::from_white_alpha(3) };
                ui.painter().rect_filled(rect, 10.0, bg);
                let img = egui::Image::from_bytes("bytes://dir.svg", include_bytes!("../../../public/icons/24px/arrow_shape_up_stack_down.svg")).tint(theme::TEXT_MUTED);
                if self.sort_reverse { img.uv(egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0))).paint_at(ui, rect.shrink(8.0)); }
                else { img.paint_at(ui, rect.shrink(8.0)); }
                if resp.clicked() { self.sort_reverse = !self.sort_reverse; }
            });
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Ok(lib) = state.library.try_read() {
                let albums: Vec<&_> = lib.albums.iter().collect();
                let buckets = groupers::generate_buckets(&albums, &self.group_key);
                for bucket in buckets {
                    let is_active = self.filter_key.as_deref() == Some(&bucket.filter_target) && self.filter_val.as_deref() == Some(&bucket.value);
                    let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
                    if response.hovered() || is_active { ui.painter().rect_filled(rect, 8.0, egui::Color32::from_white_alpha(13)); }
                    ui.painter().text(rect.left_center() + egui::vec2(12.0, 0.0), egui::Align2::LEFT_CENTER, &bucket.label, egui::FontId::proportional(14.0), if is_active { theme::TEXT_MAIN } else { theme::TEXT_MUTED });
                    ui.painter().text(rect.right_center() - egui::vec2(12.0, 0.0), egui::Align2::RIGHT_CENTER, bucket.count.to_string(), egui::FontId::monospace(12.0), theme::TEXT_MUTED);
                    if response.clicked() {
                        if is_active { self.filter_key = None; self.filter_val = None; }
                        else { self.filter_key = Some(bucket.filter_target); self.filter_val = Some(bucket.value); }
                    }
                }
            }
        });
    }
}
