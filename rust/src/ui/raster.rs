use std::collections::HashMap;
use swash::scale::ScaleContext;
use cosmic_text::{FontSystem, SwashCache};

pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct Rasterizer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub scale_context: ScaleContext,
    pub glyph_cache: HashMap<u32, AtlasRect>,
}

impl Rasterizer {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            scale_context: ScaleContext::new(),
            glyph_cache: HashMap::new(),
        }
    }
}
