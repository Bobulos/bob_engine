use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};
use crate::{rendering};
use crate::b_engine::Engine;
use std::sync::Arc;


pub static WINDOW_SIZE: (u32, u32) = (1080, 720);
pub struct App {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            engine: None,
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {

            // Create window attributes required
            let mut attributes = WindowAttributes::default();
            attributes.title = "Bob Engine".to_string();
            attributes.inner_size = Some(Size::new(Size::Physical(PhysicalSize::new(WINDOW_SIZE.0, WINDOW_SIZE.1))));
            //attributes.fullscreen = Some(Fullscreen::Borderless(None));

            let window = Arc::new(event_loop.create_window(attributes).unwrap());
            
            let mut renderer = rendering::Renderer::new();
            pollster::block_on(renderer.init(Arc::clone(&window)));
            
            let mut engine = Engine::new(renderer);
            engine.init(); // Setup ECS, etc.

            self.window = Some(window);
            self.engine = Some(engine);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if let Some(engine) = &mut self.engine {
                    engine.input.receive_input_from_app(event);
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(engine) = &mut self.engine {
                    engine.renderer.resize(physical_size.width, physical_size.height);

                    // I might actually not need this possimbly being called excessively
                    // Asumes that run doesn't catch it probably doesnt really matter too much.
                    engine.renderer.update_camera();
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(engine) = &mut self.engine {
                    engine.run();
                }
                // Tell winit to keep redrawing as fast as possible (or on VSync)
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}