use crate::renderer::Renderer;
use crate::vec::Float2;
use crate::renderer_system;
use crate::dynamic_world::DynamicWorld;
use crate::sprite::Sprite;
use crate::transform::Transform;
use std::collections::btree_map::Range;
use std::thread;
use std::time::Duration;
use std::time::Instant;
pub struct Engine<'a> {
    // We use 'a to ensure the Engine doesn't outlive the Renderer it's using
    pub renderer: &'a mut Renderer<'a>, 
    world: Option<DynamicWorld>
}

impl<'a> Engine<'a> {
    // We take a mutable reference because the engine will need 
    // to tell the renderer to clear/present/draw.
    pub fn new(renderer: &'a mut Renderer<'a>) -> Self {
        Self { renderer, world: None}
    }
    pub  fn init(&mut self) {

        self.world = Some(DynamicWorld::new());

        if let Some(w) = &mut self.world {
            // This populates the Any map with the "Templates"
            w.register_component::<Sprite>();
            w.register_component::<Transform>();
            
            println!("ECS initiated ..");

            for i in 0..100
            {
                let entity_id = w.spawn_entity();
                let transform_component = Transform::new(Float2 { x: i as f32*100f32, y: 0f32});
                w.add_component(entity_id, transform_component);
                let sprite_component  = Sprite::new(0,100,100);
                w.add_component(entity_id, sprite_component);
            }
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

            // 2. DO WORK
            self.update();
            self.render(); // Make sure this calls canvas.present()

            // 3. SLEEP
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

        let _ = self.renderer.draw_background();
        //let _ = self.renderer.draw(&self.player); // Assuming draw is a method on Renderer
        //tie in systems
        if let Some(world) = &mut self.world {
            renderer_system::render_system(world, self.renderer);
        }
        let _ = self.renderer.present();
        Ok(())
    }
}