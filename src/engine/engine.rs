use crate::coords::Float2;
use crate::core_components::sprite;
use crate::core_systems;
use crate::entities;
use crate::rendering::Renderer;
use std::ops::Add;
use std::time::Duration;
use std::time::Instant;
use std::vec;
use crate::entities::DynamicWorld;
use crate::b_engine::Input;
use crate::rendering::Instance;
use crate::core_components;
pub struct Engine {
    // We use 'a to ensure the Engine doesn't outlive the Renderer it's using
    pub renderer: Renderer, 
    world: Option<DynamicWorld>,
    pub input: Input,
    test_batch: usize,
    test_batch2: usize
}
const SPRITE_BATCH_SIZE: usize = 20000; // 2^16
impl Engine {
    // We take a mutable reference because the engine will need 
    // to tell the renderer to clear/present/draw.
    pub fn new(renderer: Renderer) -> Self {
        Self { renderer: renderer, world: None, input: Input::new(), test_batch: 0, test_batch2: 1 }
    }

    pub fn initialize(&mut self) {

        
        self.world = Some(DynamicWorld::new());
        // Load sprites and stuff
        _ = self.renderer.create_batch(
            include_bytes!("../../assets/space.jpg"),
            vec![Instance {
                    position:  [0.0, 0.0],
                    size:      [100.0, 100.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                }; 1], // Pre-allocate space for the batch size,
        );
        self.test_batch = self.renderer.create_batch(
            include_bytes!("../../assets/grass.png"),
            vec![Instance {
                    position:  [0.0, 0.0],
                    size:      [0.0, 0.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                }; SPRITE_BATCH_SIZE], // Pre-allocate space for the batch size,
        );
        self.test_batch2 = self.renderer.create_batch(
            include_bytes!("../../assets/Tux.png"),
            vec![Instance {
                    position:  [0.0, 0.0],
                    size:      [0.0, 0.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                }; SPRITE_BATCH_SIZE], // Pre-allocate space for the batch size,
        );
        let mut sprite_batch_index : usize = 0;
        if let Some(w) = &mut self.world {
            // This populates the Any map with the "Templates"
            for y in -100..0 {
                for x in -100..100 {
                    let e = w.spawn();
                    w.insert(e, entities::core_components::Transform { position: Float2 { x: x as f32, y: y as f32 }});
                    w.insert(e, entities::core_components::Sprite { texture_id: self.test_batch2 as u32, width: 1, height: 1, intra_batch_index: sprite_batch_index, batch_index: self.test_batch2 });
                    sprite_batch_index += 1;
                }
            }
            println!("ECS initiated ..");
            
        }

        

        // if let Some(world) = &mut self.world {
        //     for (entity, transform, velocity) in world.query2_mut_both::<core_components::Transform, core_components::Sprite>() {
        //         transform.x += velocity.dx;
        //         transform.y += velocity.dy;
        //     }
        // }

    }
    pub fn run(&mut self) {
        let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);
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
    const CAMERA_SPEED: f32 = 0.1;
    pub fn player_loop(&mut self) {
        if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft)) {
            self.renderer.camera.move_by(-Self::CAMERA_SPEED, 0.0);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight)) {
            self.renderer.camera.move_by(Self::CAMERA_SPEED, 0.0);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp)) {
            self.renderer.camera.move_by(0.0, Self::CAMERA_SPEED);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown)) {
            self.renderer.camera.move_by(0.0, -Self::CAMERA_SPEED);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit1)) {
            self.renderer.camera.zoom_by(1.01);
            self.renderer.update_camera();
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2)) {
            self.renderer.camera.zoom_by(0.99);
            self.renderer.update_camera();
        }
    }
    pub fn update(&mut self) {
        let target = Float2::new(100.0, 100.0);

        if let Some(world) = &mut self.world {
            for (entity, transform, sprite) in world.query2_mut_both::<core_components::Transform, core_components::Sprite>() {
                let dir = (target - transform.position).normalize_fast();
                transform.position += dir * 0.1;

                self.renderer.batches[sprite.batch_index].instances[sprite.intra_batch_index] = Instance {
                    position: transform.position.into(),
                    size: [1.0, 1.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale: [1.0, 1.0],
                };
            }
        }
        if let Some(world) =  &mut self.world {
            for i in 0..world.entity_count() {
                world.despawn(entities::Entity(i));
            }
        }


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