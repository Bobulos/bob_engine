use crate::entities::dynamic_world::DynamicWorld;
use crate::renderer::Renderer;
use crate::entities::core_components::{Sprite,Transform};
use crate::rendering::renderer;

use sdl3::{render, sys::gpu::SDL_AcquireGPUSwapchainTexture};

// #[path = "../engine//ecs/component_store.rs"]
// mod component_store;
use crate::component_store::ComponentStore;
pub fn render_system(world: &DynamicWorld, renderer: &Renderer) {
    dispatch(world, renderer);
}

fn dispatch(world: &DynamicWorld, renderer: &Renderer) {

    for (entity, pos, vel) in world.query2::<Transform, Sprite>() {
        println!("  {:?} pos={:?} vel={:?}", entity, pos, vel);
    }


    // let camera = ortho(viewport_w as f32, viewport_h as f32);

    // let cmd_buf = unsafe { SDL_AcquireGPUCommandBuffer(device) };
    // unsafe {
    //     let swapchain = SDL_AcquireGPUSwapchainTexture(cmd_buf, renderer., swapchain_texture, swapchain_texture_width, swapchain_texture_height);

    //     batcher.draw(cmd_buf, swapchain, atlas_texture, &sprites, &camera);
    //     SDL_SubmitGPUCommandBuffer(cmd_buf);
    // }
}