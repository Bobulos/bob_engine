use crate::b_engine::entities::{DynamicWorld, SystemBase};
use crate::b_engine::entities::core_components::{Sprite,Transform};

// #[path = "../engine//ecs/component_store.rs"]
// mod component_store;
pub struct RenderSystem {
    
}
impl SystemBase for RenderSystem {
    fn on_start(&self, world: &std::sync::Arc<DynamicWorld>) {
        
    }
    fn on_update(&self, world: &std::sync::Arc<DynamicWorld>) {
        
    }
    fn on_destroy(&self, world: &std::sync::Arc<DynamicWorld>) {
        
    }
}