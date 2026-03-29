use crate::core_systems;
use crate::rendering::Renderer;
use std::time::Duration;
use std::time::Instant;
use crate::entities::DynamicWorld;
use crate::b_engine::Input;
use crate::rendering::Instance;
pub struct Engine {
    // We use 'a to ensure the Engine doesn't outlive the Renderer it's using
    pub renderer: Renderer, 
    world: Option<DynamicWorld>,
    pub input: Input,
    test_batch: usize
}

impl Engine {
    // We take a mutable reference because the engine will need 
    // to tell the renderer to clear/present/draw.
    pub fn new(renderer: Renderer) -> Self {
        Self { renderer: renderer, world: None, input: Input::new(), test_batch: 0 }
    }

    pub fn initialize(&mut self) {

        self.world = Some(DynamicWorld::new());
        

        if let Some(w) = &mut self.world {
            // This populates the Any map with the "Templates"
            
            println!("ECS initiated ..");
            
        }

        // Load sprites and stuff
        self.test_batch = self.renderer.create_batch(
            include_bytes!("../../assets/test.png"),
            vec![
                Instance {
                    position:  [0.0, 0.0],
                    size:      [1.0, 1.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                },
                Instance {
                    position:  [1.0, 0.0],
                    size:      [1.0, 1.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                },
                Instance {
                    position:  [10.0, 0.0],
                    size:      [1.0, 1.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                },
            ],
        );
    }
    pub fn run(&mut self) {
        let target_frame_time = Duration::from_secs_f64(1.0 / 120.0);
        let frame_start = Instant::now();
        self.update();
        let _ = self.render();

        let elapsed = Instant::now() - frame_start;
        print!("\rFrame time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
        if elapsed > target_frame_time {
            println!("Engine running at reduced clock");
        }

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }
    // pub fn run(&mut self) {
    //     self.render();
    //     self.update();
    // }
    pub fn player_loop(&mut self) {
        if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft)) {
            self.renderer.camera.move_by(-0.05, 0.0);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight)) {
            self.renderer.camera.move_by(0.05, 0.0);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp)) {
            self.renderer.camera.move_by(0.0, 0.05);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown)) {
            self.renderer.camera.move_by(0.0, -0.05);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit1)) {
            self.renderer.camera.zoom_by(1.01);
            self.renderer.update_camera();
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2)) {
            self.renderer.camera.zoom_by(0.99);
            self.renderer.update_camera();
        }
    }
    pub fn update(&mut self) {
        self.player_loop();
        // FLUSH AT END
        self.input.flush(); // Clear per-frame input state at the start of the frame
    }

    pub fn render(&mut self) -> Result<(), String> {
        if let Some(world) = &mut self.world {
            core_systems::renderer_system::render_system(world, &self.renderer);
        }
        //let _ = self.renderer.present();
        //self.renderer.set_batch_texture(self.test_batch, include_bytes!("../../assets/test.png"));

        self.renderer.render()?;
        Ok(())
    }
}