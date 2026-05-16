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
    /// List of batch IDs per atlas (can grow as batches fill up)
    atlas_batches: Vec<Vec<usize>>,
    /// Next free instance slot per batch per atlas: atlas_next_slot[atlas_id][batch_idx]
    atlas_next_slot: Vec<Vec<usize>>,
    /// Maps atlas index -> asset name, for re-creating batches on overflow
    atlas_index_to_name: Vec<&'static str>,
}

impl SpriteBatchAllocatorSystem {
    pub fn new(renderer: Arc<RwLock<Renderer>>, included_atlases: Vec<&'static str>) -> Self {
        let mut atlas_data: HashMap<&'static str, Vec<u8>> = HashMap::new();
        let mut atlas_batches: Vec<Vec<usize>> = Vec::with_capacity(MAX_ATLASES);
        let mut atlas_next_slot: Vec<Vec<usize>> = Vec::with_capacity(MAX_ATLASES);
        let mut atlas_index_to_name: Vec<&'static str> = Vec::with_capacity(MAX_ATLASES);

        for asset_name in included_atlases.iter() {
            let asset = Asset::get(asset_name);
            if let Some(file) = asset {
                let bytes: &[u8] = &file.data;
                let mut renderer_lock = renderer.write().unwrap();
                atlas_data.insert(asset_name, bytes.to_vec());
                println!(
                    "Sprite batch allocator allocated for sprite: {:}",
                    asset_name
                );

                let batch_id = renderer_lock.create_batch(
                    bytes,
                    vec![
                        Instance {
                            position: [f32::MAX, f32::MAX],
                            size: [1.0, 1.0],
                            uv_scale: [1.0, 1.0],
                            uv_offset: [0.0, 0.0],
                        };
                        b_engine::engine::SPRITE_BATCH_SIZE
                    ],
                    crate::rendering::renderer::PipelineKey::Default,
                );

                atlas_batches.push(vec![batch_id]);
                atlas_next_slot.push(vec![0]);
            } else {
                println!("Couldn't find asset name of {:}", asset_name);
                // Still push empty entries so atlas indices stay aligned
                atlas_batches.push(vec![]);
                atlas_next_slot.push(vec![]);
            }
            atlas_index_to_name.push(asset_name);
        }

        Self {
            renderer,
            atlas_data,
            atlas_batches,
            atlas_next_slot,
            atlas_index_to_name,
        }
    }

    /// Returns `(batch_id, slot_index)` for the next free sprite slot in the given atlas.
    /// If all existing batches for this atlas are full, a new batch is created automatically.
    fn allocate_slot(&mut self, atlas_id: usize) -> (usize, usize) {
        let batches = &self.atlas_batches[atlas_id];
        let slots = &self.atlas_next_slot[atlas_id];

        // Find a batch with room
        for batch_idx in 0..batches.len() {
            if slots[batch_idx] < b_engine::engine::SPRITE_BATCH_SIZE {
                let slot = self.atlas_next_slot[atlas_id][batch_idx];
                self.atlas_next_slot[atlas_id][batch_idx] += 1;
                return (batches[batch_idx], slot); // slot is LOCAL (0..SPRITE_BATCH_SIZE)
            }
        }

        // All full — create a new batch
        println!("Atlas {} full, creating overflow batch", atlas_id);
        let asset_name = self.atlas_index_to_name[atlas_id];
        let bytes = self.atlas_data[asset_name].clone();
        let new_batch_id = {
            let mut renderer_lock = self.renderer.write().unwrap();
            renderer_lock.create_batch(
                &bytes,
                vec![
                    Instance {
                        position: [f32::MAX, f32::MAX],
                        size: [1.0, 1.0],
                        uv_scale: [1.0, 1.0],
                        uv_offset: [0.0, 0.0],
                    };
                    b_engine::engine::SPRITE_BATCH_SIZE
                ],
                crate::rendering::renderer::PipelineKey::Default,
            )
        };

        self.atlas_batches[atlas_id].push(new_batch_id);
        self.atlas_next_slot[atlas_id].push(1); // slot 0 is taken right now
        (new_batch_id, 0) // local slot 0
    }
}

impl SystemBase for SpriteBatchAllocatorSystem {
    fn on_start(&mut self, _world: &Arc<DynamicWorld>) {}

    fn on_update(&mut self, world: &Arc<DynamicWorld>) {
        let mut pending: Vec<(Entity, usize)> = Vec::new();
        world.for_each_mut::<Sprite>(|entity: Entity, sprite: &mut Sprite| {
            if sprite.index == UNASSIGNED {
                pending.push((entity, sprite.atlas_id as usize));
            }
        });

        // Allocate outside the ECS borrow
        let allocations: Vec<(Entity, usize, usize)> = pending
            .into_iter()
            .map(|(entity, atlas_id)| {
                let (batch_id, local_slot) = self.allocate_slot(atlas_id);
                (entity, batch_id, local_slot)
            })
            .collect();

        let mut alloc_iter = allocations.into_iter().peekable();
        world.for_each_mut::<Sprite>(|_entity: Entity, sprite: &mut Sprite| {
            if sprite.index == UNASSIGNED {
                if let Some((_, batch_id, local_slot)) = alloc_iter.next() {
                    sprite.batch_index = batch_id;
                    sprite.index = local_slot; // always 0..SPRITE_BATCH_SIZE
                }
            }
        });
    }

    fn on_destroy(&mut self, _world: &Arc<DynamicWorld>) {}
}
