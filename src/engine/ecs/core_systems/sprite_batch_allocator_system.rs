use crate::b_engine;
use crate::b_engine::asset_management::Asset;
use crate::b_engine::entities::Entity;
use crate::b_engine::entities::{DynamicWorld, SystemBase};
use crate::core_components::Sprite;
use crate::rendering::{Instance, Renderer};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub const MAX_ATLASES: usize = 32;
const UNASSIGNED: usize = usize::MAX;

pub struct SpriteBatchAllocatorSystem {
    pub renderer: Arc<RwLock<Renderer>>,
    pub atlas_data: HashMap<&'static str, Vec<u8>>,
    pub atlas_batch_ids: [usize; MAX_ATLASES],
    /// Next free instance slot for each atlas batch
    atlas_next_slot: [usize; MAX_ATLASES],
}

impl SpriteBatchAllocatorSystem {
    pub fn new(renderer: Arc<RwLock<Renderer>>, included_atlases: Vec<&'static str>) -> Self {
        let mut atlas_data: HashMap<&'static str, Vec<u8>> = HashMap::new();
        let mut atlas_batch_ids: [usize; MAX_ATLASES] = [0; MAX_ATLASES];

        for (index, asset_name) in included_atlases.iter().enumerate() {
            let asset = Asset::get(asset_name);
            if let Some(file) = asset {
                let bytes: &[u8] = &file.data;
                let mut renderer = renderer.write().unwrap();
                atlas_data.insert(asset_name, bytes.to_vec());
                println!(
                    "Sprite batch allocator allocated for sprite: {:}",
                    asset_name
                );
                atlas_batch_ids[index] = renderer.create_batch(
                    bytes,
                    vec![
                        Instance {
                            position: [10000.0, 100000.0],
                            size: [1.0, 1.0],
                            uv_scale: [1.0, 1.0],
                            uv_offset: [0.0, 0.0],
                        };
                        b_engine::engine::SPRITE_BATCH_SIZE
                    ],
                );
            } else {
                println!("Couldn't find asset name of {:}", asset_name);
            }
        }

        Self {
            renderer,
            atlas_data,
            atlas_batch_ids,
            atlas_next_slot: [0; MAX_ATLASES],
        }
    }
}

impl SystemBase for SpriteBatchAllocatorSystem {
    fn on_start(&mut self, _world: &Arc<DynamicWorld>) {}

    fn on_update(&mut self, world: &Arc<DynamicWorld>) {
        world.for_each_mut::<Sprite>(|_entity: Entity, sprite: &mut Sprite| {
            // Update the batch ID from the atlas mapping
            sprite.batch_index = self.atlas_batch_ids[sprite.atlas_id as usize];

            // Assign a unique slot if this sprite hasn't been allocated yet
            if sprite.index == UNASSIGNED {
                let atlas_id = sprite.atlas_id as usize;
                let slot = self.atlas_next_slot[atlas_id];
                assert!(
                    slot < b_engine::engine::SPRITE_BATCH_SIZE,
                    "Atlas {} batch is full (max {} sprites)",
                    atlas_id,
                    b_engine::engine::SPRITE_BATCH_SIZE
                );
                sprite.index = slot;
                self.atlas_next_slot[atlas_id] += 1;
            }
        });
    }

    fn on_destroy(&mut self, _world: &Arc<DynamicWorld>) {}
}
