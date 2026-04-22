use crate::b_engine::entities::{DynamicWorld, SystemBase};
use crate::b_engine::entities::core_components::{Sprite,Transform};
use crate::rendering::Renderer;
use crate::rendering::instance::Instance;
use std::sync::{Arc, RwLock};

// #[path = "../engine//ecs/component_store.rs"]
// mod component_store;
pub struct RenderSystem {
    renderer: Arc<RwLock<Renderer>>
}
impl RenderSystem {
    pub fn new(renderer: Arc<RwLock<Renderer>>) -> Self {
        Self {  
            renderer: renderer,
        }
    }
}
impl SystemBase for RenderSystem {
    fn on_start(&mut self, _world: &std::sync::Arc<DynamicWorld>) {
        
    }
    fn on_update(&mut self, world: &std::sync::Arc<DynamicWorld>) {
        world.for_each2::<Transform, Sprite>(|_entity, transform, sprite| {
            self.renderer.write().unwrap().batches[sprite.batch_index].instances[sprite.intra_batch_index] = Instance {
                position: transform.position.into(),
                size: [1.0, 1.0],
                uv_offset: [0.0, 0.0],
                uv_scale: [1.0, 1.0],
            };
            self.renderer.write().unwrap().batches[sprite.batch_index].instances[sprite.intra_batch_index] = Instance {
                position: transform.position.into(),
                size: [1.0, 1.0],
                uv_offset: [0.0, 0.0],
                uv_scale: [1.0, 1.0],
            };
        });
    }
    fn on_destroy(&mut self, _world: &std::sync::Arc<DynamicWorld>) {
        
    }
}