use crate::colors::Palette;
use crate::navigation::NavigationTab;
use egui::{Align, Button, Color32, CornerRadius, Layout, RichText, Ui, Vec2};

pub fn render_navigation_bar(
    ui: &mut Ui,
    active_tab: &mut NavigationTab,
    palette: &Palette,
) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space(8.0);

        let home_text = RichText::new("H").size(15.0).strong();
        let home_btn = Button::new(home_text)
            .min_size(Vec2::new(32.0, 32.0))
            .corner_radius(CornerRadius::same(8))
            .fill(if *active_tab == NavigationTab::Home {
                palette.ok400
            } else {
                Color32::TRANSPARENT
            });

        if ui.add(home_btn).clicked() {
            *active_tab = NavigationTab::Home;
        }

        ui.add_space(6.0);

        let queue_text = RichText::new("Q").size(15.0).strong();
        let queue_btn = Button::new(queue_text)
            .min_size(Vec2::new(32.0, 32.0))
            .corner_radius(CornerRadius::same(8))
            .fill(if *active_tab == NavigationTab::Queue {
                palette.ok400
            } else {
                Color32::TRANSPARENT
            });

        if ui.add(queue_btn).clicked() {
            *active_tab = NavigationTab::Queue;
        }
    });
}
