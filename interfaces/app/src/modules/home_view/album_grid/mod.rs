pub mod grid_controller;
pub mod layout;
pub mod scroll;

use self::grid_controller::GridController;
use crate::colors::Palette;
use crate::config::AppConfig;
use crate::library::collection::AlbumSummary;
use crate::library::prewarmer::Prewarmer;
use egui::{
    Color32, CornerRadius, Event, FontId, Painter, Pos2, Rect, Response, Sense, Shadow, Ui, Vec2,
};

const KEYBOARD_SCROLL_SPEED: f32 = 0.2;
const TEXT_FADE_DISTANCE: f32 = 40.0;

fn process_grid_input(ui: &Ui, ctrl: &mut GridController) {
    ui.input(|i| {
        for event in &i.raw.events {
            if let Event::MouseWheel { delta, .. } = event {
                if delta.y < -0.01 {
                    ctrl.scroll_step(1.0);
                } else if delta.y > 0.01 {
                    ctrl.scroll_step(-1.0);
                }
            }
        }

        let mut key_delta = 0.0;
        if i.key_down(egui::Key::J) || i.key_down(egui::Key::ArrowDown) {
            key_delta += KEYBOARD_SCROLL_SPEED;
        }
        if i.key_down(egui::Key::K) || i.key_down(egui::Key::ArrowUp) {
            key_delta -= KEYBOARD_SCROLL_SPEED;
        }
        if key_delta != 0.0 {
            ctrl.scroll_row(key_delta);
        }
    });
}

fn fit_text(painter: &Painter, text: &str, font_id: &FontId, max_width: f32) -> String {
    let galley = painter.layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE);
    if galley.size().x <= max_width {
        return text.to_string();
    }

    let ellipsis = "...";
    let chars: Vec<char> = text.chars().collect();
    let mut low = 0_usize;
    let mut high = chars.len();
    let mut best_len = 0_usize;

    while low <= high {
        let mid = (low + high) / 2;
        let candidate: String = chars[..mid].iter().collect::<String>() + ellipsis;
        let candidate_galley =
            painter.layout_no_wrap(candidate, font_id.clone(), Color32::WHITE);

        if candidate_galley.size().x <= max_width {
            best_len = mid;
            low = mid.saturating_add(1);
        } else {
            if mid == 0 {
                break;
            }
            high = mid.saturating_sub(1);
        }
    }

    chars[..best_len].iter().collect::<String>() + ellipsis
}

fn get_or_layout_text(
    painter: &Painter,
    ctrl: &mut GridController,
    album: &AlbumSummary,
    config: &AppConfig,
) -> (String, String) {
    if let Some(cached) = ctrl.text_cache.get(&album.id) {
        return cached.clone();
    }

    let cover_size =
        f32::from(u16::try_from(config.album_grid.album_card.cover.size).unwrap_or(200));
    let text_config = &config.album_grid.album_card.text;

    let font_title = FontId::proportional(text_config.title.size);
    let title_fit = fit_text(painter, &album.title, &font_title, cover_size);

    let font_artist = FontId::proportional(text_config.albumartist.size);
    let artist_fit = fit_text(painter, &album.artist, &font_artist, cover_size);

    let result = (title_fit, artist_fit);
    ctrl.text_cache.insert(album.id.clone(), result.clone());
    result
}

fn render_card_shadows(painter: &Painter, rect: Rect) {
    let shadow_1 = Shadow {
        offset: [0, 0],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(76),
    };
    let shadow_2 = Shadow {
        offset: [0, 0],
        blur: 8,
        spread: 0,
        color: Color32::from_black_alpha(25),
    };
    let shadow_3 = Shadow {
        offset: [0, 0],
        blur: 6,
        spread: 0,
        color: Color32::from_black_alpha(25),
    };

    painter.add(shadow_1.as_shape(rect, CornerRadius::ZERO));
    painter.add(shadow_2.as_shape(rect, CornerRadius::ZERO));
    painter.add(shadow_3.as_shape(rect, CornerRadius::ZERO));
}

fn render_card_text(
    painter: &Painter,
    title: &str,
    artist: &str,
    pos: Pos2,
    metadata_top: f32,
    config: &AppConfig,
    palette: &Palette,
) {
    let opacity = if metadata_top < TEXT_FADE_DISTANCE {
        (metadata_top / TEXT_FADE_DISTANCE).max(0.0)
    } else {
        1.0
    };

    if opacity <= 0.0 {
        return;
    }

    let text_config = &config.album_grid.album_card.text;
    let title_color = palette.ok100.gamma_multiply(opacity);
    let artist_color = palette.ok100.gamma_multiply(opacity * 0.7);

    let font_title = FontId::proportional(text_config.title.size);
    painter.text(pos, egui::Align2::LEFT_TOP, title, font_title, title_color);

    let title_lh = (text_config.title.size * 1.2).round();
    let artist_pos = Pos2::new(pos.x, pos.y + title_lh + text_config.spacing.middle);
    let font_artist = FontId::proportional(text_config.albumartist.size);
    painter.text(
        artist_pos,
        egui::Align2::LEFT_TOP,
        artist,
        font_artist,
        artist_color,
    );
}

pub fn render_album_grid(
    ui: &mut Ui,
    ctrl: &mut GridController,
    prewarmer: &mut Prewarmer,
    config: &AppConfig,
    palette: &Palette,
    albums: &[AlbumSummary],
    on_focus: &mut impl FnMut(String),
) -> Response {
    ctrl.albums = albums.to_vec();
    ctrl.layout.config = config.album_grid.clone();

    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    ctrl.layout.container_width = rect.width();
    ctrl.viewport_height = rect.height();

    let dpr = ui.ctx().pixels_per_point();
    ctrl.update(dpr);

    if response.hovered() {
        process_grid_input(ui, ctrl);
    }

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::ZERO, palette.ok200);

    let cover_size = ctrl.layout.card_size();
    let gap_x = ctrl.layout.gap_x();
    let gap_y = ctrl.layout.gap_y();
    let start_x = rect.min.x + ((rect.width() - ctrl.layout.grid_width()) / 2.0).floor();
    let text_gap = if config.album_grid.album_card.text.enable {
        config.album_grid.album_card.text.spacing.top
    } else {
        0.0
    };

    let scroll_y = ctrl.scroll.current_y;
    for row in ctrl.virtual_rows() {
        let row_y = rect.min.y + row.y - scroll_y;
        if row_y + ctrl.layout.row_height() < rect.min.y - 100.0
            || row_y > rect.max.y + 100.0
        {
            continue;
        }

        for (col_idx, album) in row.data.iter().enumerate() {
            let col_u16 = u16::try_from(col_idx).unwrap_or(0);
            let x = f32::from(col_u16).mul_add(cover_size + gap_x, start_x);
            let y = row_y + gap_y;
            let cover_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(cover_size));

            render_card_shadows(&painter, cover_rect);

            if let Some(hash) = &album.cover_hash {
                if let Some(tex) = prewarmer.get(hash) {
                    painter.image(
                        tex.id(),
                        cover_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    painter.rect_filled(cover_rect, CornerRadius::ZERO, palette.ok300);
                }
            } else {
                painter.rect_filled(cover_rect, CornerRadius::ZERO, palette.ok300);
            }

            if config.album_grid.album_card.text.enable {
                let text_pos = Pos2::new(x, y + cover_size + text_gap);
                let meta_top = (row.y - scroll_y) + gap_y + cover_size + text_gap;
                let (title, artist) = get_or_layout_text(&painter, ctrl, album, config);
                render_card_text(
                    &painter,
                    &title,
                    &artist,
                    text_pos,
                    meta_top,
                    config,
                    palette,
                );
            }

            if response.clicked()
                && ui.input(|i| {
                    i.pointer
                        .interact_pos()
                        .is_some_and(|p| cover_rect.contains(p))
                })
            {
                on_focus(album.id.clone());
            }
        }
    }

    response
}
