use crate::config::AppConfig;
use crate::expand_path;
use crate::server::library::Library;
use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

slint::slint! {
    struct Album {
        id: string,
        title: string,
        artist: string,
        cover: image,
        active: bool,
    }

    struct AlbumRow {
        index: int,
        y: length,
        data: [Album],
    }

    component AlbumCard inherits Rectangle {
        in property <Album> album;
        in property <length> row-y;
        in property <length> render-y;

        callback clicked();

        width: 190px;
        height: 249px;

        property <length> absolute-y: root.row-y - root.render-y;
        property <length> metadata-top: root.absolute-y + 217px; // 16px (gap-y) + 190px (cover) + 11px (text-gap)
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
                opacity: root.text-opacity;

                VerticalLayout {
                    spacing: 2px;
                    y: -root.clip-amount;
                    
                    Text {
                        text: root.album.title;
                        color: root.album.active ? #ffffff : #ffffff;
                        font-size: 14px;
                        font-family: "Inter";
                        font-weight: 400;
                        overflow: elide;
                    }
                    Text {
                        text: root.album.artist;
                        color: #cccccc;
                        font-size: 12px;
                        font-family: "Inter";
                        font-weight: 400;
                        overflow: elide;
                    }
                }
            }
        }
    }

    component Row inherits Rectangle {
        in property <AlbumRow> row-data;
        in property <length> render-y;
        callback item-clicked(string);

        y: root.row-data.y - root.render-y;
        height: 249px;
        width: 100%;

        HorizontalLayout {
            spacing: 30px;
            alignment: center;

            for album in root.row-data.data : AlbumCard {
                album: album;
                row-y: root.row-data.y;
                render-y: root.render-y;
                clicked => { root.item-clicked(album.id); }
            }
        }
    }

    export component AppWindow inherits Window {
        background: #111111;
        width: 1024px;
        height: 768px;
        title: "Vellum";

        in property <[AlbumRow]> virtual-rows;
        in property <length> render-y;
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
                    item-clicked(id) => { root.item_clicked(id); }
                }
            }
        }
    }
}

#[derive(Clone)]
struct AlbumData {
    id: String,
    title: String,
    artist: String,
    // [CRITICAL FIX]: Pre-load and store the decoded `slint::Image` to avoid disk I/O and decoding every 16ms
    cover: slint::Image,
}

pub fn run() -> anyhow::Result<()> {
    let (config, _, _) = AppConfig::load()?;
    let library_root = expand_path(&config.storage.library_root).canonicalize()?;
    
    let thumb_size = config.theme.as_ref().map_or(190, |t| t.thumbnail_size);
    let thumb_dir = config.storage.thumbnail_cache_folder.clone().map(|p| expand_path(&p).join(format!("{}px", thumb_size)));

    let mut library = Library::new(library_root.clone());
    library.scan();

    log::info!("Pre-loading UI covers into memory to guarantee smooth scrolling...");

    let library_albums: Vec<AlbumData> = library.albums.into_iter().map(|a| {
        let mut target_img = library_root.join(&a.id).join(&a.album_data.info.cover_path);
        
        if let Some(td) = &thumb_dir {
            let hash = &a.album_data.info.cover_hash;
            if !hash.is_empty() {
                let cached = td.join(format!("{}.png", hash));
                if cached.exists() {
                    target_img = cached;
                }
            }
        }

        let img = if target_img.exists() {
            slint::Image::load_from_path(&target_img).unwrap_or_default()
        } else {
            slint::Image::default()
        };

        AlbumData {
            id: a.id,
            title: a.album_data.album,
            artist: a.album_data.albumartist,
            cover: img,
        }
    }).collect();

    let ui = AppWindow::new().unwrap();

    let current_y = Rc::new(std::cell::Cell::new(0.0f32));
    let target_slot = Rc::new(std::cell::Cell::new(0));

    // [CRITICAL FIX]: Memoization storage to prevent re-creating the entire virtual DOM tree every frame.
    // Re-allocating the `virtual_rows` model forces Slint to layout/draw text repeatedly causing massive stutter.
    let last_bounds = Rc::new(std::cell::Cell::new((usize::MAX, usize::MAX, 0usize)));
    let cached_rows = Rc::new(std::cell::RefCell::new(Vec::new()));

    let row_height = 249.0;
    let damping = 0.18;
    let top_offset = 4.0;
    let gap_x = 30.0;
    let card_size = 190.0;

    let ui_weak = ui.as_weak();
    
    ui.on_scroll_slot({
        let target_slot = target_slot.clone();
        move |delta| {
            let mut slot = target_slot.get() + delta;
            if slot < 0 { slot = 0; }
            target_slot.set(slot);
        }
    });

    ui.on_item_clicked(|id| {
        log::info!("Album clicked: {}", id);
    });

    let _timer = slint::Timer::default();
    _timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
        let ui = ui_weak.unwrap();
        
        let container_width = ui.get_container_width() as f32;
        let viewport_height = ui.get_viewport_height() as f32;

        let mut cols = ((container_width - 40.0 + gap_x) / (card_size + gap_x)).floor() as usize;
        if cols < 1 { cols = 1; }

        let bounds = last_bounds.get();

        // Only group chunks if window resizes / column counts mutate
        if cols != bounds.2 {
            *cached_rows.borrow_mut() = library_albums.chunks(cols).map(|c| c.to_vec()).collect();
        }

        let rows = cached_rows.borrow();
        let total_rows = rows.len();

        let visible_rows = (viewport_height / row_height).ceil() as usize;
        let max_slots = if total_rows >= visible_rows { (total_rows - visible_rows + 1) as i32 } else { 0 };

        let mut current_slot = target_slot.get();
        if current_slot > max_slots {
            current_slot = max_slots;
            target_slot.set(current_slot);
        }

        let target_y = current_slot as f32 * row_height;
        let mut y = current_y.get();

        let diff = target_y - y;
        if diff.abs() > 0.01 {
            y += diff * damping;
        } else {
            y = target_y;
        }

        current_y.set(y);

        // Update the raw float to GPU translation binding on EVERY frame
        ui.set_render_y(y); 

        let buffer = 4;
        let start_idx = ((y / row_height).floor() as isize - buffer).max(0) as usize;
        let end_idx = (((y + viewport_height) / row_height).ceil() as isize + buffer).max(0) as usize;
        let end_idx = end_idx.min(total_rows.saturating_sub(1));

        // [CRITICAL FIX]: Only replace Slin's virtual row models when the threshold buffers actually cross boundaries
        if start_idx != bounds.0 || end_idx != bounds.1 || cols != bounds.2 {
            let mut virtual_rows = Vec::new();
            for i in start_idx..=end_idx {
                if i < rows.len() {
                    let row_y = (i as f32 * row_height) + top_offset;

                    let mut album_data = Vec::new();
                    for album in &rows[i] {
                        album_data.push(Album {
                            id: SharedString::from(&album.id),
                            title: SharedString::from(&album.title),
                            artist: SharedString::from(&album.artist),
                            cover: album.cover.clone(), // Uses the preloaded memory cache (reference/Arc bound)
                            active: false,
                        });
                    }

                    virtual_rows.push(AlbumRow {
                        index: i as i32,
                        y: row_y,
                        data: Rc::new(VecModel::from(album_data)).into(),
                    });
                }
            }

            ui.set_virtual_rows(Rc::new(VecModel::from(virtual_rows)).into());
            last_bounds.set((start_idx, end_idx, cols));
        }
    });

    ui.run()?;
    Ok(())
}
