use crate::coords::Float2;
use crate::core_systems;
use crate::entities;
use crate::rendering::Renderer;
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
const SPRITE_BATCH_SIZE: usize = 1024*4; // 2^16
impl Engine {
    // We take a mutable reference because the engine will need 
    // to tell the renderer to clear/present/draw.
    pub fn new(renderer: Renderer) -> Self {
        Self { renderer: renderer, world: None, input: Input::new(), test_batch: 0, test_batch2: 1 }
    }

    pub fn initialize(&mut self) {

        
        self.world = Some(DynamicWorld::new());
        _ = self.renderer.create_batch(
            include_bytes!("../../assets/space.jpg"),
            vec![Instance {
                    position:  [0.0, 0.0],
                    size:      [0.0, 0.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                }; 1], // Pre-allocate space for the batch size,
        );

        let mut spawned = 0;
        if let Some(world) = &mut self.world {
            for b in 0..50 {
                let mut sprite_batch_index : usize = 0;
                let batch = self.renderer.create_batch(
                include_bytes!("../../assets/Tux.png"),
                vec![Instance {
                        position:  [0.0, 0.0],
                        size:      [0.0, 0.0],
                        uv_offset: [0.0, 0.0],
                        uv_scale:  [1.0, 1.0],
                    }; SPRITE_BATCH_SIZE], // Pre-allocate space for the batch size,
                );
                for y in 0..SPRITE_BATCH_SIZE {
                    spawned += 1;
                    let e = world.spawn();
                    world.insert(e, entities::core_components::Transform { position: Float2 { x: y as f32, y: y as f32 }});
                    world.insert(e, entities::core_components::Sprite { texture_id: batch as u32, width: 1, height: 1, 
                        intra_batch_index: sprite_batch_index, batch_index: batch });
                    sprite_batch_index += 1;
                }
            }
        }
        print!("Spawned {} entities", spawned);
        if let Some(w) = &mut self.world {
            // This populates the Any map with the "Templates"

            println!("ECS initiated ..");
        }

        let terrain_png = include_bytes!("../../assets/tiles.png");
        let tree_png = include_bytes!("../../assets/grass.png");
        let my_map: Vec<u32> = vec![0; 10000];

        let background = self.renderer.create_tilemap(terrain_png, &my_map, 512, 512, 32);
        let trees = self.renderer.create_tilemap(tree_png, &my_map, 512, 512, 100);

        self.renderer.tilemaps[trees].move_by(0.0, 0.5);
        self.renderer.tilemaps[trees].flush_position(self.renderer.queue());
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
    const CAMERA_SPEED: f32 = 1.178657;
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


        // Updates the sprites positions on the gpu
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


        self.player_loop();
        // FLUSH AT END
        self.input.flush(); // Clear per-frame input state at the start of the frame
    }

    pub fn render(&mut self) -> Result<(), String> {
        if let Some(world) = &mut self.world {
            core_systems::renderer_system::render_system(world, &self.renderer);
        }
        self.renderer.render()?;
        Ok(())
    }
}