use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::component_store::ComponentStore;
pub struct DynamicWorld {
    // Key: The Type itself (e.g., the "ID" of the Position struct)
    // Value: A Boxed (heap-allocated) ComponentStore of that type
    // Points to contius memory chunks on the heap
    // With component allocations
    storages: HashMap<TypeId, Box<dyn Any>>,
    entities_count: usize
}

impl DynamicWorld {
    pub fn new() -> Self {
        Self { storages: HashMap::new(), entities_count: 0 }
    }
    pub fn register_component<T: 'static>(&mut self) {
        let storage: ComponentStore<T> = ComponentStore::new();
        self.storages.insert(TypeId::of::<T>(), Box::new(storage));
    }
    pub fn add_component<T: 'static>(&mut self, entity_id: usize, component: T) {
        // Get the storage (mutably this time!)
        if let Some(boxed_storage) = self.storages.get_mut(&TypeId::of::<T>()) {
            
            // Downcast to the concrete storage type
            if let Some(storage) = boxed_storage.downcast_mut::<ComponentStore<T>>() {
                
                // Since we are inside the crate, we can access the private fields
                // or use a public method on ComponentStore to insert
                storage.insert(entity_id, component);
            }
        }
    }
    pub fn spawn_entity(&mut self) -> usize {
        let id = self.entities_count;
        self.entities_count += 1;
        id
    }
    pub fn get_storage<T: 'static>(&self) -> Option<&ComponentStore<T>> {
        self.storages.get(&TypeId::of::<T>())?
            .downcast_ref::<ComponentStore<T>>()
    }
}