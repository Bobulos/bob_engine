// use sdl3::image::LoadTexture; // Trait for loading textures directly
// use sdl3::rect::Rect;
// use std::error::Error;
// use std::path::Path;
#[path ="engine/renderer/renderer.rs"]
pub mod renderer;
use renderer::Renderer;
#[path ="engine/math/vec.rs"]
pub mod coords;
#[path ="engine/engine.rs"]
pub mod engine;
use  engine::Engine;
#[path ="test/player.rs"]
pub mod player;
#[path = "engine/ecs/component_store.rs"]
pub mod component_store;
#[path = "engine/ecs/mod.rs"]
pub mod entities;
#[path = "engine/ecs/core_systems/mod.rs"]
pub mod core_systems;
#[path = "engine/ecs/core_systems/core_components/mod.rs"]
pub mod core_components;
#[path = "engine/renderer/mod.rs"]
pub mod rendering;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("bob_engine running...");
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let window = video.window("Bob Engine", 1080, 720).build()?;
    

    //let creator = canvas.texture_creator();
    
    // let mut manager = TextureCache::new(&creator);
    // manager.load("assets/test.png")?;
    
    let mut renderer = Renderer::new(window);
    
    let mut engine = Engine::new(renderer);
    engine.init();

    let event_pump = sdl.event_pump()?;
    engine.run(event_pump);
    Ok(())
}