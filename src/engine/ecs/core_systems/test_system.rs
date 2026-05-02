use crate::b_engine::entities::{DynamicWorld, Entity, SystemBase};
use crate::b_engine::entities::core_components::Transform;
use std::sync::Arc;
use crate::b_engine;
// #[path = "../engine//ecs/component_store.rs"]
// mod component_store;
pub struct TestSystem {
}
impl TestSystem {
    pub fn new() -> Self {
        Self {
            
        }
    }
}
const GRAVITY: f32 = 9.8;
impl SystemBase for TestSystem {
    fn on_start(&mut self, _world: &Arc<DynamicWorld>) {
        
    }
    fn on_update(&mut self, world: &Arc<DynamicWorld>) {
        world.for_each_mut::<Transform>(|_entity: Entity, transform: &mut Transform| {
            transform.position.y -= GRAVITY * b_engine::engine::FIXED_DT; // Apply gravity to the y position
        });
    }
    fn on_destroy(&mut self, _world: &Arc<DynamicWorld>) {
        
    }
}