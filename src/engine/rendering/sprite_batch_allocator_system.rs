use crate::b_engine::asset_management::Asset;
use crate::b_engine::entities::{DynamicWorld, SystemBase};
use crate::rendering::Renderer;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub const MAX_ATLASES: usize = 32;
pub struct SpriteBatchAllocatorSystem {
    // This system will manage the allocation of sprites into batches for efficient rendering
    // It will keep track of which sprites are in which batches and handle adding/removing sprites from batches as needed
    // It will also handle resizing batches when they become full
    pub renderer: Arc<RwLock<Renderer>>,

    // DO NOT USE A WHOLE LOT AT RUNTIME THIS MF IS SLOW AS HELL ON HEAP TO AVOID BLOAT SINGLE THREAD ACCESS ONLY
    pub atlas_data: HashMap<&'static str, Vec<u8>>, // Store the raw data of the atlases for reference when adding sprites to batches
}
impl SpriteBatchAllocatorSystem {
    /// When specifying limit to 32 as is the max atlas.
    /// Creates a batch for each atlas and adds it to the renderer.
    /// The system will then manage which sprites go into which batch based on their texture_id.
    pub fn new(renderer: Arc<RwLock<Renderer>>, included_atlases: Vec<&'static str>) -> Self {
        Self {
            renderer,
            atlas_data: load_atlas_data(included_atlases),
        }
    }
}
fn load_atlas_data(included_atlases: Vec<&'static str>) -> HashMap<&'static str, Vec<u8>> {
    let mut atlas_data: HashMap<&'static str, Vec<u8>> = HashMap::new();
    for asset_name in included_atlases.iter() {
        println!("Sprite batch allocator loading {:}", asset_name);
        let file = Asset::get(asset_name).unwrap();
        let bytes: &[u8] = &file.data;
        atlas_data.insert(asset_name, bytes.to_vec());
    }
    atlas_data
}
impl SystemBase for SpriteBatchAllocatorSystem {
    fn on_start(&mut self, _world: &Arc<DynamicWorld>) {}
    fn on_update(&mut self, _world: &Arc<DynamicWorld>) {}
    fn on_destroy(&mut self, _world: &Arc<DynamicWorld>) {}
}
