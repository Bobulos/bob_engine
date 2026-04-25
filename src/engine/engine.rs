use crate::b_engine;
use crate::b_engine::entities::SystemGroup;
use crate::b_engine::entities::entities::Entities;
use crate::b_engine::entities::system_group::SystemGroupThreading;
use crate::coords::Float2;
use crate::b_engine::entities;
use crate::core_systems;
use crate::rendering::Renderer;
use std::sync::RwLock;
use std::time::Duration;
use std::time::Instant;
use std::{vec,sync::Arc};

use crate::b_engine::entities::DynamicWorld;
use crate::b_engine::Input;
use crate::rendering::Instance;
use crate::b_engine::asset_management::Asset;
pub struct Engine {
    pub renderer: Arc<RwLock<Renderer>>, 
    pub input: Input,
    pub entities: Entities
}
pub const MAIN_WORLD: &str = "main";
pub const RENDER_GROUP: &str = "render_group";
pub const SPRITE_BATCH_SIZE: usize = 1024*4; // 2^10
pub const FIXED_DT: f32 = 1.0/60.0; // 2^14
impl Engine {
    // We take a mutable reference because the engine will need 
    // to tell the renderer to clear/present/draw.s
    pub fn new(renderer: Renderer) -> Self {
        Self { 
            renderer: Arc::new( RwLock::new(renderer)), 
            input: Input::new(), 
            entities: Entities::new() 
        }
    }

    pub fn init(&mut self) {
        self.debug_list_assets();
        self.setup_world();
        self.setup_renderer();
        self.setup_systems();
        println!("Engine initialized");
    }

    fn debug_list_assets(&self) {
        for file in Asset::iter() {
            println!("{}", file.as_ref());
        }
    }

    fn setup_world(&mut self) {
        self.entities.add_world(MAIN_WORLD, Arc::new(DynamicWorld::new()));
        b_engine::entities::system_bootstrap::bootstrap(&self);
    }

    fn setup_renderer(&mut self) {
        self.setup_background();
        self.setup_sprites();
        self.setup_tilemap();
    }

    fn setup_background(&mut self) {
        self.renderer
            .write()
            .unwrap()
            .create_batch(
                include_bytes!("../../assets/space.jpg"),
                vec![Instance {
                    position:  [0.0, 0.0],
                    size:      [0.0, 0.0],
                    uv_offset: [0.0, 0.0],
                    uv_scale:  [1.0, 1.0],
                }; 1],
            );
    }

    fn setup_sprites(&mut self) {
        let file = Asset::get("tree.png").unwrap();
        let bytes: &[u8] = &file.data;
        let mut spawned = 0;

        for _ in 0..1 {
            let batch = self.renderer
                .write()
                .unwrap()
                .create_batch(
                    bytes,
                    vec![Instance {
                        position:  [0.0, 0.0],
                        size:      [0.0, 0.0],
                        uv_offset: [0.0, 0.0],
                        uv_scale:  [1.0, 1.0],
                    }; SPRITE_BATCH_SIZE],
                );

            let world = self.entities.get_world(MAIN_WORLD).unwrap();
            for y in 0..SPRITE_BATCH_SIZE {
                spawned += 1;
                let e = world.spawn();
                world.insert(e, entities::core_components::Transform {
                    position: Float2 { x: y as f32, y: y as f32 },
                });
                world.insert(e, entities::core_components::Sprite {
                    texture_id: batch as u32,
                    width: 1,
                    height: 1,
                    intra_batch_index: y,
                    batch_index: batch,
                });
            }
        }

        println!("Spawned {} entities", spawned);
    }

    fn setup_tilemap(&mut self) {
        let tilemap = [0u8; 64 * 64];


        let file = Asset::get("grass.png").unwrap();
        let bytes: &[u8] = &file.data;
        let test = Asset::get("test.png").unwrap();
        let test_bytes: &[u8] = &test.data;
        // Acquire the lock once to do all tilemap work — avoids the deadlock
        // that occurs when holding a write guard and calling .queue() via a
        // second write() on the same RwLock in the same expression.
        let mut renderer = self.renderer.write().unwrap();
        let trees = renderer.create_tilemap(test_bytes, &tilemap, 64, 64, 32);
        renderer.tilemaps[trees].move_by(0.0, -0.5);
        let queue = renderer.queue();
        renderer.tilemaps[trees].flush_position(queue);
        let trees = renderer.create_tilemap(bytes, &tilemap, 64, 64, 100);
        renderer.tilemaps[trees].move_by(0.0, 0.0);
        let queue = renderer.queue();
        renderer.tilemaps[trees].flush_position(queue);


        let trees = renderer.create_tilemap(bytes, &tilemap, 64, 64, 100);
        renderer.tilemaps[trees].move_by(65.0, 0.0);
        let queue = renderer.queue();
        renderer.tilemaps[trees].flush_position(queue);

        let trees = renderer.create_tilemap(bytes, &tilemap, 64, 64, 100);
        renderer.tilemaps[trees].move_by(65.0, 65.0);
        let queue = renderer.queue();
        renderer.tilemaps[trees].flush_position(queue);

        let trees = renderer.create_tilemap(bytes, &tilemap, 64, 64, 100);
        renderer.tilemaps[trees].move_by(0.0, 65.0);
        let queue = renderer.queue();
        renderer.tilemaps[trees].flush_position(queue);
    }

    fn setup_systems(&mut self) {
        println!("Initializing system groups");

        let fetched_world = self.entities.get_world(MAIN_WORLD).unwrap();
        self.entities.add_system_group(RENDER_GROUP, SystemGroup::new(fetched_world, SystemGroupThreading::Parallel));
        
        let group = self.entities.get_system_group_mut(RENDER_GROUP).unwrap();
        let _rendering_system = group.register_system(Box::new(
            core_systems::render_system::RenderSystem::new(Arc::clone(&self.renderer))
        ));
    }
    pub fn run(&mut self) {
        let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);
        let frame_start = Instant::now();
        self.update();
        self.render().expect("Renderer fatal error");

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
    const CAMERA_SPEED: f32 = 0.1;
    pub fn player_loop(&mut self) {
        if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft)) {
            self.renderer.write().unwrap().camera.move_by(-Self::CAMERA_SPEED, 0.0);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight)) {
            self.renderer.write().unwrap().camera.move_by(Self::CAMERA_SPEED, 0.0);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp)) {
            self.renderer.write().unwrap().camera.move_by(0.0, Self::CAMERA_SPEED);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown)) {
            self.renderer.write().unwrap().camera.move_by(0.0, -Self::CAMERA_SPEED);
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit1)) {
            self.renderer.write().unwrap().camera.zoom_by(1.01);
            self.renderer.write().unwrap().update_camera();
        } if self.input.get_key_down(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2)) {
            self.renderer.write().unwrap().camera.zoom_by(0.99);
            self.renderer.write().unwrap().update_camera();
        }
    }
    pub fn update(&mut self) {
        self.update_entities();

        self.player_loop();
        // FLUSH AT END
        self.input.flush(); // Clear per-frame input state at the start of the frame
    }
    pub fn render(&mut self) -> Result<(), String> {
        self.renderer.write().unwrap().render().expect("Fatal error from renderer");
        Ok(())
    }
    fn update_entities(&mut self) {
        self.entities.update_system_groups();
    }
}