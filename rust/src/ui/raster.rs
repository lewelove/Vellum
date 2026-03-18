use std::collections::HashMap;
use swash::scale::ScaleContext;
use cosmic_text::{FontSystem, SwashCache};

pub struct AtlasRect {
    pub _x: u32,
    pub _y: u32,
    pub _w: u32,
    pub _h: u32,
}

pub struct Rasterizer {
    pub _font_system: FontSystem,
    pub _swash_cache: SwashCache,
    pub _scale_context: ScaleContext,
    pub _glyph_cache: HashMap<u32, AtlasRect>,
}

impl Rasterizer {
    pub fn new() -> Self {
        Self {
            _font_system: FontSystem::new(),
            _swash_cache: SwashCache::new(),
            _scale_context: ScaleContext::new(),
            _glyph_cache: HashMap::new(),
        }
    }
}
