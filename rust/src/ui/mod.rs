pub mod engine;
pub mod physics;
pub mod raster;

use anyhow::Result;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use std::sync::Arc;

use crate::config::AppConfig;
use crate::expand_path;
use crate::server::library::Library;

pub fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

#[derive(Default)]
struct App {
    state: Option<engine::State>,
    physics: physics::PhysicsEngine,
    window: Option<Arc<Window>>,
    library: Option<Library>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let (config, _, _) = AppConfig::load().expect("Failed to load config");
        let library_root = expand_path(&config.storage.library_root)
            .canonicalize()
            .expect("Invalid library root");
        
        let mut lib = Library::new(library_root);
        lib.scan();
        
        let window_attrs = Window::default_attributes().with_title("Vellum");
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        self.window = Some(window.clone());

        let state = pollster::block_on(engine::State::new(window.clone(), &lib, &config));
        self.state = Some(state);
        self.library = Some(lib);
        
        if let Some(s) = self.state.as_mut() {
            self.physics.update_layout(s.size.width, s.size.height);
        }
        
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (state, physics, window, library) = match (
            self.state.as_mut(), 
            &mut self.physics, 
            self.window.as_ref(), 
            self.library.as_ref()
        ) {
            (Some(s), p, Some(w), Some(l)) => (s, p, w, l),
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
                physics.update_layout(physical_size.width, physical_size.height);
                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let line_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => f64::from(y) * 120.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y,
                };
                
                let row_count = (library.albums.len() as f32 / physics.cols as f32).ceil() as usize;
                let visible_rows = (physics.container_height as f32 / physics.row_height).ceil() as usize;
                let max_slots = (row_count.saturating_sub(visible_rows) + 1) as f64;
                
                physics.scroll(-line_delta, max_slots);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                physics.tick();
                state.write_instances(physics, library);
                state.update(physics);
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
                window.request_redraw();
            }
            _ => {}
        }
    }
}
