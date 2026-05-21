use crate::b_engine::entities::core_components::Transform;
use crate::b_engine::entities::{DynamicWorld, Entity, SystemBase};
use crate::float2::Float2;
use crate::{b_engine, core_components};
use std::sync::Arc;
// #[path = "../engine//ecs/component_store.rs"]
// mod component_store;
pub struct TestSystem {
    spawned: u32,
    targets: Vec<Float2>,
}
impl TestSystem {
    pub fn new() -> Self {
        Self {
            spawned: 0,
            targets: Vec::new(),
        }
    }
}
const GRAVITY: f32 = 9.8;
const ENTITY_COUNT: usize = 10000;
impl SystemBase for TestSystem {
    fn on_start(&mut self, world: &Arc<DynamicWorld>) {}
    fn on_update(&mut self, world: &Arc<DynamicWorld>) {
        for _ in 0..20 {
            self.spawned += 1;
            let spawned = self.spawned as f32 * 0.01;
            let position = Float2 {
                x: spawned * f32::sin(spawned),
                y: spawned * f32::cos(spawned),
            };

            let e = world.create_entity();
            world.add_component(
                e,
                core_components::Sprite {
                    visible: true,
                    batch_index: 0,
                    index: usize::MAX,
                    atlas_id: 0,
                    width: 1,
                    height: 1,
                },
            );

            world.add_component(e, core_components::Transform { position });
        }

        const SPEED: f32 = 0.01;
        world.for_each_mut::<Transform>(|entity: Entity, transform: &mut Transform| {
            let dir = Float2::ZERO - transform.position;
            dir.normalize();
            transform.position += dir * SPEED;
        });
    }

    fn on_destroy(&mut self, _world: &Arc<DynamicWorld>) {}
}
