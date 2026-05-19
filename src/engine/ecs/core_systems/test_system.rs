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
    fn on_start(&mut self, _world: &Arc<DynamicWorld>) {}
    fn on_update(&mut self, world: &Arc<DynamicWorld>) {
        // colections
        let mut consumed = 0;
        for _ in 0..10 {
            self.spawned += 1;
            let e = world.spawn();
            world.insert(
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

            world.insert(
                e,
                core_components::Transform {
                    position: Float2::new(self.spawned as f32, self.spawned as f32),
                },
            );
        }

        //let mut targets: [Float2; ENTITY_COUNT] = [Float2::ZERO; ENTITY_COUNT];
        //let mut targets: Vec<Float2> = Vec::new();
        self.targets.clear();
        world.for_each_mut::<Transform>(|entity: Entity, transform: &mut Transform| {
            self.targets.push(transform.position);
            consumed += 1;
        });
        const FORCE: f32 = 0.001;
        world.for_each_mut::<Transform>(|oponents: Entity, transform: &mut Transform| {
            let mut avg = Float2::ZERO;
            for i in 0..consumed {
                let dif = (transform.position - self.targets[i]);
                let lngth = dif.length();
                if (lngth > 0.0) {
                    avg += dif / lngth;
                }
            }
            avg.normalize();
            transform.position += avg * b_engine::engine::FIXED_DT * FORCE;
        });
    }
    fn on_destroy(&mut self, _world: &Arc<DynamicWorld>) {}
}
