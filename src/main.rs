// use sdl3::image::LoadTexture; // Trait for loading textures directly
// use sdl3::rect::Rect;
// use std::error::Error;
// use std::path::Path;
#[path ="engine/mod.rs"]
pub mod b_engine;
#[path ="engine/math/vec.rs"]
pub mod coords;
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
#[path = "engine/rendering/mod.rs"]
pub mod rendering;
#[path = "engine/rendering/tilemap/mod.rs"]
pub mod tilemap;


use winit::event_loop::EventLoop;
use crate::app::App;
mod app;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("bob_engine running...");
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}