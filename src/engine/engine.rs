use crate::core_systems;
use crate::rendering::Renderer;
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
        Self { renderer: renderer, world: None}
    }
    pub  fn initialize(&mut self) {

        self.world = Some(DynamicWorld::new());
        

        if let Some(w) = &mut self.world {
            // This populates the Any map with the "Templates"
            
            println!("ECS initiated ..");
            
        }
    }
    pub fn run(&mut self) {
        let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);
        let frame_start = Instant::now();
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
    // pub fn run(&mut self) {
    //     self.render();
    //     self.update();
    // }
    pub fn update(&mut self) {
        
    }

    pub fn render(&mut self) -> Result<(), String> {
        if let Some(world) = &mut self.world {
            core_systems::renderer_system::render_system(world, &self.renderer);
        }
        //let _ = self.renderer.present();
        self.renderer.render();
        Ok(())
    }
}