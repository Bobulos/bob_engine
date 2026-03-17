// use sdl3::image::LoadTexture; // Trait for loading textures directly
// use sdl3::rect::Rect;
// use std::error::Error;
// use std::path::Path;
use crate::renderer::texture_cache::TextureCache;
#[path ="core_systems/core_components/sprite.rs"]
pub mod sprite;
#[path ="core_systems/core_components/transform.rs"]
pub mod transform;
#[path ="core_systems/renderer_system.rs"]
pub mod renderer_system;
#[path ="renderer/renderer.rs"]
pub mod renderer;
use renderer::Renderer;
#[path ="math/vec.rs"]
pub mod vec;
#[path ="engine/engine.rs"]
pub mod engine;
use  engine::Engine;
#[path ="test/player.rs"]
pub mod player;
#[path = "engine/ecs/component_store.rs"]
pub mod component_store;
#[path = "engine/ecs/dynamic_world.rs"]
pub mod dynamic_world;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("bob_engine running...");
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let window = video.window("Bob Engine", 1080, 720).build()?;
    
    let canvas = window.into_canvas();
    let creator = canvas.texture_creator(); // <--- The Parent
    
    let mut manager = TextureCache::new(&creator);
    manager.load("assets/test.png")?;
    
    let mut renderer = Renderer::new(canvas, &manager);
    
    let mut engine = Engine::new(&mut renderer);
    engine.init();

    let event_pump = sdl.event_pump()?;
    engine.run(event_pump);
    Ok(())
}