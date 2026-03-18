use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use winit::window::Window;
use crate::ui::physics::PhysicsEngine;
use crate::ui::raster::Rasterizer;
use crate::server::library::scanner::Library;
use crate::config::AppConfig;
use crate::expand_path;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Globals {
    pub viewport_size: [f32; 2],
    pub scroll_y: f32,
    pub padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct AlbumInstance {
    pub position: [f32; 2],
    pub tex_index: i32,
    pub is_text: i32,
}

pub struct State {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    pub globals_buffer: wgpu::Buffer,
    pub globals_bind_group: wgpu::BindGroup,
    
    pub grid_pipeline: wgpu::RenderPipeline,
    
    pub instance_buffer: wgpu::Buffer,
    pub num_covers: u32,
    pub num_texts: u32,

    pub album_tex_bind_group: wgpu::BindGroup,
    pub text_tex_bind_group: wgpu::BindGroup,
    pub album_id_to_tex: std::collections::HashMap<String, i32>,
    pub album_id_to_text_tex: std::collections::HashMap<String, i32>,

    pub _rasterizer: Rasterizer,
}

impl State {
    pub async fn new(window: Arc<Window>, library: &Library, app_config: &AppConfig) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        }).await.unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals Buffer"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let (album_tex_view, album_id_to_tex) = Self::create_texture_array(&device, &queue, library, app_config);
        let (text_tex_view, album_id_to_text_tex) = Self::create_text_array(&device, &queue, library);

        let tex_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Texture Bind Group Layout"),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let album_tex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &tex_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&album_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Album Texture Bind Group"),
        });

        let text_tex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &tex_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&text_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Text Texture Bind Group"),
        });

        let grid_wgsl = std::fs::read_to_string("rust/src/ui/shaders/grid.wgsl").expect("Missing grid.wgsl");

        let grid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(grid_wgsl.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&globals_bind_group_layout, &tex_bind_group_layout],
            push_constant_ranges: &[],
        });

        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &grid_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<AlbumInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Sint32, 2 => Sint32],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &grid_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<AlbumInstance>() as u64) * 10000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            globals_buffer,
            globals_bind_group,
            grid_pipeline,
            instance_buffer,
            num_covers: 0,
            num_texts: 0,
            album_tex_bind_group,
            text_tex_bind_group,
            album_id_to_tex,
            album_id_to_text_tex,
            _rasterizer: Rasterizer::new(),
        }
    }

    fn create_texture_array(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        library: &Library,
        app_config: &AppConfig,
    ) -> (wgpu::TextureView, std::collections::HashMap<String, i32>) {
        let max_supported = device.limits().max_texture_array_layers;
        let layer_count = (library.albums.len() as u32).min(max_supported).max(1);
        
        let size = 190;
        let texture_extent = wgpu::Extent3d { 
            width: size, 
            height: size, 
            depth_or_array_layers: layer_count 
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Album Texture Array"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let thumb_root = app_config.storage.thumbnail_cache_folder.as_deref().map(expand_path).unwrap_or_default();
        let thumb_dir = thumb_root.join("190px");

        let mut id_map = std::collections::HashMap::new();
        let mut count = 0;

        for a in library.albums.iter().take(layer_count as usize) {
            let hash = &a.album_data.info.cover_hash;
            let id = a.id.clone();
            let thumb_path = thumb_dir.join(format!("{}.png", hash));
            let fallback_path = library.root.join(&id).join(&a.album_data.info.cover_path);
            
            let path = if thumb_path.exists() { thumb_path } else { fallback_path };
            if let Ok(img) = image::open(path) {
                let rgba = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3).to_rgba8();
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: count as u32 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    rgba.as_raw(),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * size),
                        rows_per_image: Some(size),
                    },
                    wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
                );
                id_map.insert(id, count as i32);
                count += 1;
            }
        }

        (texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        }), id_map)
    }

    fn create_text_array(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        library: &Library,
    ) -> (wgpu::TextureView, std::collections::HashMap<String, i32>) {
        let max_supported = device.limits().max_texture_array_layers;
        let layer_count = (library.albums.len() as u32).min(max_supported).max(1);
        let w = 190;
        let h = 32;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Texture Array"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: layer_count },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut id_map = std::collections::HashMap::new();
        let mut font_system = cosmic_text::FontSystem::new();
        let mut swash_context = swash::scale::ScaleContext::new();

        for i in 0..layer_count {
            let album = &library.albums[i as usize];
            let pixmap = crate::ui::text::render_text_blob(
                &album.album_data.album,
                &album.album_data.albumartist,
                &mut font_system,
                &mut swash_context,
                1.0
            );

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: i },
                    aspect: wgpu::TextureAspect::All,
                },
                pixmap.data(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * w),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            id_map.insert(album.id.clone(), i as i32);
        }

        (texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        }), id_map)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn write_instances(&mut self, physics: &PhysicsEngine, library: &Library) {
        let (start, end) = physics.get_visible_range(library.albums.len());
        let count = end - start;
        let mut cover_instances = Vec::with_capacity(count);
        let mut text_instances = Vec::with_capacity(count);
        
        for i in start..end {
            let album = &library.albums[i];
            let tex_idx = self.album_id_to_tex.get(&album.id).copied().unwrap_or(-1);
            let text_idx = self.album_id_to_text_tex.get(&album.id).copied().unwrap_or(-1);
            let pos = physics.get_item_pos(i);
            
            cover_instances.push(AlbumInstance {
                position: [pos[0], pos[1] + 16.0],
                tex_index: tex_idx,
                is_text: 0,
            });

            text_instances.push(AlbumInstance {
                position: [pos[0], pos[1] + 16.0 + 190.0 + 11.0],
                tex_index: text_idx,
                is_text: 1,
            });
        }
        
        self.num_covers = cover_instances.len() as u32;
        self.num_texts = text_instances.len() as u32;

        let mut final_buffer = cover_instances;
        final_buffer.extend(text_instances);
        
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&final_buffer));
    }

    pub fn update(&mut self, physics: &PhysicsEngine) {
        let globals = Globals {
            viewport_size: [self.size.width as f32, self.size.height as f32],
            scroll_y: physics.current_y as f32,
            padding: 0.0,
        };
        self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.066, g: 0.066, b: 0.066, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if self.num_covers > 0 {
                rp.set_pipeline(&self.grid_pipeline);
                rp.set_bind_group(0, &self.globals_bind_group, &[]);
                
                // Segment 1: Covers
                rp.set_bind_group(1, &self.album_tex_bind_group, &[]);
                rp.set_vertex_buffer(0, self.instance_buffer.slice(..));
                rp.draw(0..6, 0..self.num_covers);

                // Segment 2: Text (offset by size of num_covers in the vertex buffer)
                let text_offset = (self.num_covers as u64) * (std::mem::size_of::<AlbumInstance>() as u64);
                rp.set_bind_group(1, &self.text_tex_bind_group, &[]);
                rp.set_vertex_buffer(0, self.instance_buffer.slice(text_offset..));
                rp.draw(0..6, 0..self.num_texts);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
