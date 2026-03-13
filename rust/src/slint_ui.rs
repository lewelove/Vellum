use crate::config::AppConfig;
use crate::expand_path;
use crate::server::library::Library;
use slint::{Model, SharedString, VecModel};
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

pub fn run() -> anyhow::Result<()> {
    let (config, _, _) = AppConfig::load()?;
    let library_root = expand_path(&config.storage.library_root).canonicalize()?;
    
    let thumb_size = config.theme.as_ref().map_or(190, |t| t.thumbnail_size);
    let thumb_dir = config.storage.thumbnail_cache_folder.clone().map(|p| expand_path(&p).join(format!("{}px", thumb_size)));

    let mut library = Library::new(library_root.clone());
    library.scan();

    log::info!("Pre-loading UI covers into memory to guarantee smooth scrolling...");

    let library_albums: Vec<Album> = library.albums.into_iter().map(|a| {
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

        Album {
            id: SharedString::from(&a.id),
            title: SharedString::from(&a.album_data.album),
            artist: SharedString::from(&a.album_data.albumartist),
            cover: img,
            active: false,
        }
    }).collect();

    let ui = AppWindow::new().unwrap();

    const POOL_SIZE: usize = 18;
    let physical_rows: Vec<AlbumRow> = (0..POOL_SIZE).map(|_| AlbumRow {
        index: -1,
        data: Rc::new(VecModel::default()).into(),
    }).collect();
    
    let physical_model = Rc::new(VecModel::from(physical_rows));
    ui.set_virtual_rows(physical_model.clone().into());

    let current_y = Rc::new(std::cell::Cell::new(0.0f32));
    let target_slot = Rc::new(std::cell::Cell::new(0));
    let last_time = Rc::new(std::cell::Cell::new(std::time::Instant::now()));

    let last_cols = Rc::new(std::cell::Cell::new(0usize));
    let last_container_width = Rc::new(std::cell::Cell::new(-1.0f32));
    let logical_rows = Rc::new(std::cell::RefCell::new(Vec::new()));

    let row_height = 249.0;
    let scroll_speed = 12.0f32;
    let gap_x = 30.0;
    let card_size = 190.0;

    let mapped_rows = Rc::new(std::cell::RefCell::new(vec![-1i32; POOL_SIZE]));

    let ui_weak = ui.as_weak();
    
    ui.on_scroll_slot({
        let target_slot = target_slot.clone();
        move |delta| {
            let mut slot = target_slot.get() + delta;
            if slot < 0 { slot = 0; }
            target_slot.set(slot);
        }
    });

    ui.on_item_clicked(|_id| {});

    let _timer = slint::Timer::default();
    _timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
        let ui = ui_weak.unwrap();
        
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_time.get()).as_secs_f32().min(0.1);
        last_time.set(now);

        let container_width = ui.get_container_width() as f32;
        let viewport_height = ui.get_viewport_height() as f32;

        if (container_width - last_container_width.get()).abs() > 0.01 {
            last_container_width.set(container_width);

            let mut cols = ((container_width - 40.0 + gap_x) / (card_size + gap_x)).floor() as usize;
            if cols < 1 { cols = 1; }

            let grid_width = (cols as f32 * card_size) + ((cols.saturating_sub(1)) as f32 * gap_x);
            ui.set_grid_width(grid_width);

            if cols != last_cols.get() {
                let chunks: Vec<slint::ModelRc<Album>> = library_albums.chunks(cols).map(|c| {
                    Rc::new(VecModel::from(c.to_vec())).into()
                }).collect();
                *logical_rows.borrow_mut() = chunks;
                
                for i in 0..POOL_SIZE {
                    physical_model.set_row_data(i, AlbumRow {
                        index: -1,
                        data: Rc::new(VecModel::default()).into(),
                    });
                }
                last_cols.set(cols);
                mapped_rows.borrow_mut().fill(-1);
            }
        }

        let rows = logical_rows.borrow();
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
            let t = 1.0 - (-scroll_speed * dt).exp();
            y += diff * t;
        } else {
            y = target_y;
        }

        current_y.set(y);
        ui.set_render_y(y); 

        let buffer = 4;
        let start_idx = ((y / row_height).floor() as isize - buffer).max(0) as usize;
        let end_idx = (((y + viewport_height) / row_height).ceil() as isize + buffer).max(0) as usize;
        let end_idx = end_idx.min(total_rows.saturating_sub(1));

        let mut cache = mapped_rows.borrow_mut();
        for i in start_idx..=end_idx {
            if i < total_rows {
                let physical_idx = i % POOL_SIZE;
                let target_index = i as i32;
                
                if cache[physical_idx] != target_index {
                    physical_model.set_row_data(physical_idx, AlbumRow {
                        index: target_index,
                        data: rows[i].clone(),
                    });
                    cache[physical_idx] = target_index;
                }
            }
        }
    });

    ui.run()?;
    Ok(())
}
