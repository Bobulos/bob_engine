use crate::b_engine::entities::SystemGroup;
use crate::b_engine::entities::system_group::SystemGroupThreading;
use crate::b_engine::{Engine, engine};
use crate::rendering;

use std::sync::Arc;

pub const EARLY_ORDER: i16 = -1000;
pub const DEFAULT_ORDER: i16 = 0;
pub const LATE_ORDER: i16 = 1000;

// This module contains functions to bootstrap the engine systems.
// Bootstraps rendering allocators ect...
pub fn bootstrap_systems(engine: &mut Engine) {
    bootstrap_render_system(engine);
    bootstrap_sprite_batch_allocator(engine);
}

fn bootstrap_render_system(engine: &mut Engine) {
    let group = engine
        .entities
        .get_system_group_mut(engine::RENDER_GROUP)
        .unwrap();
    let _rendering_system = group.register_system(
        Box::new(rendering::RenderSystem::new(Arc::clone(&engine.renderer))),
        LATE_ORDER,
    );
    let _atlasses = [""; rendering::sprite_batch_allocator_system::MAX_ATLASES];
    let _rendering_system = group.register_system(
        Box::new(
            rendering::sprite_batch_allocator_system::SpriteBatchAllocatorSystem::new(
                Arc::clone(&engine.renderer),
                engine::INCLUDED_TEXTURES.to_vec(),
            ),
        ),
        EARLY_ORDER,
    );
}
fn bootstrap_sprite_batch_allocator(engine: &mut Engine) {
    let fetched_world = engine.entities.get_world(engine::MAIN_WORLD).unwrap();
    engine.entities.add_system_group(
        engine::RENDER_GROUP,
        SystemGroup::new(fetched_world, SystemGroupThreading::Parallel),
    );

    // let fetched_world = self.entities.get_world(engine::MAIN_WORLD).unwrap();
    // self.entities.add_system_group(
    //     "test_group",
    //     SystemGroup::new(fetched_world, SystemGroupThreading::Parallel),
    // );
    // let group = self.entities.get_system_group_mut("test_group").unwrap();
    // group.register_system(Box::new(core_systems::test_system::TestSystem::new()));
    // // initialize them jhons
    // self.entities.start_system_groups();
}
