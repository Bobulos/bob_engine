use std::sync::{Arc, RwLock};
use crate::rendering::{Renderer, Instance};
use crate::b_engine::entities::{DynamicWorld, Entity, SystemBase, SystemGroup};
use crate::b_engine::asset_management::Asset;

pub const MAX_ATLASES: usize = 32;
pub struct SpriteBatchAllocatorSystem {
    // This system will manage the allocation of sprites into batches for efficient rendering
    // It will keep track of which sprites are in which batches and handle adding/removing sprites from batches as needed
    // It will also handle resizing batches when they become full
    pub renderer: Arc<RwLock<Renderer>>,
    included_atlases: Vec<&'static str>,
}
impl SpriteBatchAllocatorSystem {
    /// When specifying limit to 32 as is the max atlas.
    /// Creates a batch for each atlas and adds it to the renderer. 
    /// The system will then manage which sprites go into which batch based on their texture_id.
    pub fn new(renderer: Arc<RwLock<Renderer>>, included_atlases: Vec<&'static str>) -> Self {
        for asset_name in included_atlases.iter() {
            let file = Asset::get(asset_name).unwrap();
            let bytes: &[u8] = &file.data;
            let mut renderer = renderer.write().unwrap();
            // This jhon needs to have some instances
            renderer.create_batch(bytes);
        }
        Self {
            renderer,
            included_atlases
        }
    }
}
impl SystemBase for SpriteBatchAllocatorSystem {
    fn on_start(&mut self, _world: &Arc<DynamicWorld>) {

    }
    fn on_update(&mut self, world: &Arc<DynamicWorld>) {

    }
    fn on_destroy(&mut self, _world: &Arc<DynamicWorld>) {
        
    }
}