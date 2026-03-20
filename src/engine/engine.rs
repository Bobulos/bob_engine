use crate::core_systems;
use crate::renderer::Renderer;
use crate::coords::Float2;
use std::time::Duration;
use std::time::Instant;
use crate::entities::DynamicWorld;
pub struct Engine {
    // We use 'a to ensure the Engine doesn't outlive the Renderer it's using
    pub renderer: Renderer, 
    world: Option<DynamicWorld>
}

impl Engine {
    // We take a mutable reference because the engine will need 
    // to tell the renderer to clear/present/draw.
    pub fn new(renderer: Renderer) -> Self {
        Self { renderer, world: None}
    }
    pub  fn init(&mut self) {

        self.world = Some(DynamicWorld::new());
        
        self.renderer.init();

        if let Some(w) = &mut self.world {
            // This populates the Any map with the "Templates"
            
            println!("ECS initiated ..");
            
        }
    }
    pub fn run(&mut self, mut event_pump: sdl3::EventPump) {
        let mut is_running = true;
        let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);

        while is_running {
            let frame_start = Instant::now();
            // 1. POLL HERE (This is the only place it needs to happen)
            for event in event_pump.poll_iter() {
                match event {
                    sdl3::event::Event::Quit {..} => is_running = false,
                    _ => {}
                }
            }
            
            self.update();
            let _ = self.render();

            let elapsed = Instant::now() - frame_start;
            if elapsed > target_frame_time {
                println!("Engine running at reduced clock");
            }

            let elapsed = frame_start.elapsed();
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            }
        }
    }
    // pub fn run(&mut self) {
    //     self.render();
    //     self.update();
    // }
    pub fn update(&mut self) {
        
    }

    pub fn render(&mut self) -> Result<(), String> {

        //let _ = self.renderer.draw_background();
        //let _ = self.renderer.draw(&self.player); // Assuming draw is a method on Renderer
        //tie in systems
        if let Some(world) = &mut self.world {
            core_systems::renderer_system::render_system(world, &self.renderer);
        }
        //let _ = self.renderer.present();
        Ok(())
    }
}