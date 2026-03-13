use crate::config::AppConfig;
use crate::expand_path;
use crate::server::library::Library;
use slint::{Model, SharedString, VecModel, SharedPixelBuffer, Rgba8Pixel};
use std::rc::Rc;
use rayon::prelude::*;
use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Wrap, Weight};
use tiny_skia::{Pixmap, Color};

slint::slint! {
    export struct Album {
        id: string,
        cover: image,
        text_blob: image,
        active: bool,
    }

    export struct AlbumRow {
        index: int,
        data: [Album],
    }

    component AlbumCard inherits Rectangle {
        in property <Album> album;
        in property <length> absolute-y;

        callback clicked();

        width: 190px;
        height: 249px;

        property <length> metadata-top: root.absolute-y + 217px;
        property <float> text-opacity: max(0.0, min(1.0, (root.metadata-top / 1px) / 40.0));
        property <length> clip-amount: max(0.0, -root.metadata-top / 1px) * 1px;

        VerticalLayout {
            spacing: 0px;
            padding-top: 16px;

            TouchArea {
                width: 190px;
                height: 190px;
                clicked => { root.clicked(); }

                Rectangle {
                    width: 100%;
                    height: 100%;
                    background: #323232;
                    drop-shadow-color: rgba(0, 0, 0, 0.4);
                    drop-shadow-blur: 8px;
                    drop-shadow-offset-y: 0px;

                    Image {
                        source: root.album.cover;
                        width: 100%;
                        height: 100%;
                        image-fit: cover;
                    }
                }
            }

            Rectangle {
                height: 11px;
            }

            Rectangle {
                height: 32px;
                clip: true;
                opacity: root.text_opacity;
                background: #323232;

                Image {
                    source: root.album.text_blob;
                    y: -root.clip-amount;
                    width: 190px;
                    height: 32px;
                    image-fit: fill;
                }
            }
        }
    }

    component Row inherits Rectangle {
        in property <AlbumRow> row-data;
        in property <length> render-y;
        in property <length> grid-width;
        in property <length> container-width;
        callback item-clicked(string);

        property <length> absolute-y: (root.row-data.index * 249px) + 4px - root.render-y;

        visible: root.row-data.index >= 0;
        x: (root.container-width - root.grid-width) / 2;
        y: root.absolute-y;
        height: 249px;
        width: root.grid-width;

        HorizontalLayout {
            spacing: 30px;
            alignment: start;

            for album in root.row-data.data : AlbumCard {
                album: album;
                absolute-y: root.absolute-y;
                clicked => { root.item-clicked(album.id); }
            }
        }
    }

    export component AppWindow inherits Window {
        background: #111111;
        preferred-width: 1024px;
        preferred-height: 768px;
        title: "Vellum";

        in property <[AlbumRow]> virtual-rows;
        in property <length> render-y;
        in property <length> grid-width;
        out property <length> viewport-height: self.height;
        out property <length> container-width: self.width;

        callback scroll_slot(int);
        callback item_clicked(string);

        property <float> wheel-accum: 0.0;

        forward-focus: key-handler;

        key-handler := FocusScope {
            key-pressed(event) => {
                if (event.text == "j" || event.text == "\u{001f}") {
                    root.scroll_slot(1);
                    return accept;
                }
                if (event.text == "k" || event.text == "\u{001e}") {
                    root.scroll_slot(-1);
                    return accept;
                }
                return reject;
            }
        }

        TouchArea {
            width: 100%;
            height: 100%;
            
            scroll-event(event) => {
                root.wheel-accum += event.delta-y / 1px;
                if (abs(root.wheel-accum) > 40.0) {
                    if (root.wheel-accum < 0.0) {
                        root.scroll_slot(1);
                    } else {
                        root.scroll_slot(-1);
                    }
                    root.wheel-accum = 0.0;
                }
                return accept;
            }

            Rectangle {
                width: 100%;
                height: 100%;
                clip: true;
                background: #323232;

                for row in root.virtual-rows : Row {
                    row-data: row;
                    render-y: root.render-y;
                    grid-width: root.grid-width;
                    container-width: root.container-width;
                    item-clicked(id) => { root.item_clicked(id); }
                }
            }
        }
    }
}

fn truncate_with_ellipsis(
    text: &str,
    font_system: &mut cosmic_text::FontSystem,
    metrics: Metrics,
    max_width: f32,
    attrs: &Attrs,
) -> String {
    let mut buffer = Buffer::new(font_system, metrics);
    buffer.set_size(font_system, None, None);
    buffer.set_text(font_system, text, attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(font_system, false);

    let current_width = buffer.layout_runs().next().map(|r| r.line_w).unwrap_or(0.0);
    if current_width <= max_width {
        return text.to_string();
    }

    let mut current_text = text.to_string();
    while !current_text.is_empty() {
        current_text.pop();
        let display = format!("{}…", current_text);
        buffer.set_text(font_system, &display, attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);
        let w = buffer.layout_runs().next().map_or(0.0, |r| r.line_w);
        if w <= max_width {
            return display;
        }
    }
    String::new()
}

fn blend_channel(src_lin: f32, dst_lin: f32, mask: u8) -> u8 {
    let alpha = mask as f32 / 255.0;
    let out_lin = src_lin * alpha + dst_lin * (1.0 - alpha);
    (out_lin.powf(1.0 / 2.2) * 255.0).round() as u8
}

fn render_text_blob(
    title: &str,
    artist: &str,
    font_system: &mut cosmic_text::FontSystem,
    swash_cache: &mut cosmic_text::SwashCache,
    scale: f32,
) -> Pixmap {
    let width = (190.0 * scale).round() as u32;
    let height = (32.0 * scale).round() as u32;
    let mut pixmap = Pixmap::new(width, height).unwrap();
    
    let gamma: f32 = 2.2;
    let bg_color: [f32; 3] = [50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0];
    let bg_lin: [f32; 3] = [bg_color[0].powf(gamma), bg_color[1].powf(gamma), bg_color[2].powf(gamma)];
    
    pixmap.fill(Color::from_rgba(bg_color[0], bg_color[1], bg_color[2], 1.0).unwrap());

    let mut render_line = |text: &str, y_off: f32, size: f32, weight: Weight, color: [f32; 3]| {
        let src_lin: [f32; 3] = [color[0].powf(gamma), color[1].powf(gamma), color[2].powf(gamma)];
        let attrs = Attrs::new().family(Family::Name("Inter")).weight(weight);
        let metrics = Metrics::new(size * scale, (size + 3.0) * scale);
        
        let truncated = truncate_with_ellipsis(text, font_system, metrics, width as f32, &attrs);
        
        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_size(font_system, Some(width as f32), Some(height as f32));
        buffer.set_wrap(font_system, Wrap::None);
        buffer.set_text(font_system, &truncated, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);

        if let Some(run) = buffer.layout_runs().next() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0.0, (run.line_y + y_off).round()), 1.0);
                if let Some(image) = swash_cache.get_image(font_system, physical_glyph.cache_key) {
                    let x_start = physical_glyph.x + image.placement.left;
                    let y_start = physical_glyph.y - image.placement.top;

                    let data = pixmap.data_mut();
                    
                    for row in 0..image.placement.height as i32 {
                        for col in 0..image.placement.width as i32 {
                            let px = x_start + col;
                            let py = y_start + row;
                            if px < 0 || px >= width as i32 || py < 0 || py >= height as i32 { continue; }

                            let idx = (py as usize * width as usize + px as usize) * 4;
                            let mask_idx = row as usize * image.placement.width as usize + col as usize;

                            match image.content {
                                cosmic_text::SwashContent::SubpixelMask => {
                                    let m_idx = mask_idx * 3;
                                    data[idx] = blend_channel(src_lin[0], bg_lin[0], image.data[m_idx]);
                                    data[idx+1] = blend_channel(src_lin[1], bg_lin[1], image.data[m_idx+1]);
                                    data[idx+2] = blend_channel(src_lin[2], bg_lin[2], image.data[m_idx+2]);
                                    data[idx+3] = 255;
                                }
                                cosmic_text::SwashContent::Mask => {
                                    let m = image.data[mask_idx];
                                    data[idx] = blend_channel(src_lin[0], bg_lin[0], m);
                                    data[idx+1] = blend_channel(src_lin[1], bg_lin[1], m);
                                    data[idx+2] = blend_channel(src_lin[2], bg_lin[2], m);
                                    data[idx+3] = 255;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    };

    render_line(title, 0.0, 14.0, Weight::MEDIUM, [1.0, 1.0, 1.0]);
    render_line(artist, 18.0 * scale, 12.0, Weight::NORMAL, [204.0/255.0, 204.0/255.0, 204.0/255.0]);

    pixmap
}

pub fn run() -> anyhow::Result<()> {
    let (config, _, _) = AppConfig::load()?;
    let library_root = expand_path(&config.storage.library_root).canonicalize()?;
    let thumb_size = config.theme.as_ref().map_or(190, |t| t.thumbnail_size);
    let thumb_dir = config.storage.thumbnail_cache_folder.clone().map(|p| expand_path(&p).join(format!("{}px", thumb_size)));

    let mut library = Library::new(library_root.clone());
    library.scan();

    let ui = AppWindow::new().unwrap();
    let scale_factor = ui.window().scale_factor();

    let mut root_db = cosmic_text::fontdb::Database::new();
    root_db.load_system_fonts();

    log::info!("Generating LCD-Optimized Text Blobs (Scale: {})...", scale_factor);

    let album_data_vec: Vec<_> = library.albums.par_iter().map_init(
        || {
            let font_system = cosmic_text::FontSystem::new_with_locale_and_db("en-US".to_string(), root_db.clone());
            let swash_cache = cosmic_text::SwashCache::new();
            (font_system, swash_cache)
        },
        |(font_system, swash_cache), a| {
            let mut target_img = library_root.join(&a.id).join(&a.album_data.info.cover_path);
            if let Some(td) = &thumb_dir {
                let hash = &a.album_data.info.cover_hash;
                if !hash.is_empty() {
                    let cached = td.join(format!("{}.png", hash));
                    if cached.exists() { target_img = cached; }
                }
            }
            let pixmap = render_text_blob(&a.album_data.album, &a.album_data.albumartist, font_system, swash_cache, scale_factor);
            (a.id.clone(), target_img, pixmap)
        }
    ).collect();

    let library_albums: Vec<Album> = album_data_vec.into_iter().map(|(id, target_img, pixmap)| {
        let cover = if target_img.exists() { slint::Image::load_from_path(&target_img).unwrap_or_default() } else { slint::Image::default() };
        let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(pixmap.width(), pixmap.height());
        buffer.make_mut_bytes().copy_from_slice(pixmap.data());
        Album { id: SharedString::from(&id), cover, text_blob: slint::Image::from_rgba8(buffer), active: false }
    }).collect();

    const POOL_SIZE: usize = 18;
    let physical_model = Rc::new(VecModel::from((0..POOL_SIZE).map(|_| AlbumRow { index: -1, data: Rc::new(VecModel::default()).into() }).collect::<Vec<_>>()));
    ui.set_virtual_rows(physical_model.clone().into());

    let current_y = Rc::new(std::cell::Cell::new(0.0f32));
    let target_slot = Rc::new(std::cell::Cell::new(0));
    let last_time = Rc::new(std::cell::Cell::new(std::time::Instant::now()));
    let last_container_width = Rc::new(std::cell::Cell::new(-1.0f32));
    let last_cols = Rc::new(std::cell::Cell::new(0usize));
    let logical_rows: Rc<std::cell::RefCell<Vec<slint::ModelRc<Album>>>> = Rc::new(std::cell::RefCell::new(Vec::new()));
    let mapped_rows = Rc::new(std::cell::RefCell::new(vec![-1i32; POOL_SIZE]));

    ui.on_scroll_slot({
        let target_slot = target_slot.clone();
        move |delta| { let s = target_slot.get() + delta; target_slot.set(s.max(0)); }
    });

    let ui_weak = ui.as_weak();
    let _timer = slint::Timer::default();
    _timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
        let ui = ui_weak.unwrap();
        let dt = std::time::Instant::now().duration_since(last_time.get()).as_secs_f32().min(0.1);
        last_time.set(std::time::Instant::now());

        let cw = ui.get_container_width() as f32;
        if (cw - last_container_width.get()).abs() > 0.01 {
            last_container_width.set(cw);
            let cols = ((cw - 40.0 + 30.0) / (190.0 + 30.0)).floor() as usize;
            let cols = cols.max(1);
            ui.set_grid_width((cols as f32 * 190.0) + (cols.saturating_sub(1) as f32 * 30.0));
            if cols != last_cols.get() {
                *logical_rows.borrow_mut() = library_albums.chunks(cols).map(|c| Rc::new(VecModel::from(c.to_vec())).into()).collect();
                last_cols.set(cols);
                mapped_rows.borrow_mut().fill(-1);
            }
        }

        let rows = logical_rows.borrow();
        let visible_rows = (ui.get_viewport_height() as f32 / 249.0).ceil() as usize;
        let max_s = if rows.len() >= visible_rows { (rows.len() - visible_rows + 1) as i32 } else { 0 };
        let mut slot = target_slot.get();
        if slot > max_s { slot = max_s; target_slot.set(slot); }

        let target_y = slot as f32 * 249.0;
        let mut y = current_y.get();
        let diff = target_y - y;
        if diff.abs() > 0.01 { y += diff * (1.0 - (-12.0 * dt).exp()); } else { y = target_y; }
        current_y.set(y);
        ui.set_render_y(y);

        let start = ((y / 249.0).floor() as isize - 4).max(0) as usize;
        let end = (((y + ui.get_viewport_height() as f32) / 249.0).ceil() as isize + 4).max(0) as usize;
        let end = end.min(rows.len().saturating_sub(1));

        let mut cache = mapped_rows.borrow_mut();
        for i in start..=end {
            if i < rows.len() {
                let p_idx = i % POOL_SIZE;
                if cache[p_idx] != i as i32 {
                    physical_model.set_row_data(p_idx, AlbumRow { index: i as i32, data: rows[i].clone() });
                    cache[p_idx] = i as i32;
                }
            }
        }
    });

    ui.run()?;
    Ok(())
}
