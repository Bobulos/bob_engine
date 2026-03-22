use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::component_store::ComponentStore;
use crate::entities::query::{NoFilter, QueryFilter};

// ---------------------------------------------------------------------------
// Entity handle (newtype around usize for type-safety)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------
pub struct DynamicWorld {
    storages: HashMap<TypeId, Box<dyn Any>>,
    alive: Vec<bool>,
    entities_count: usize,
}

impl DynamicWorld {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
            alive: Vec::new(),
            entities_count: 0,
        }
    }

    // ------------------------------------------------------------------
    // Entity management
    // ------------------------------------------------------------------

    pub fn spawn(&mut self) -> Entity {
        let id = self.entities_count;
        self.entities_count += 1;
        if id >= self.alive.len() {
            self.alive.resize(id + 1, false);
        }
        self.alive[id] = true;
        Entity(id)
    }

    pub fn despawn(&mut self, entity: Entity) {
        if let Some(slot) = self.alive.get_mut(entity.0) {
            *slot = false;
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive.get(entity.0).copied().unwrap_or(false)
    }

    pub fn entity_count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    // ------------------------------------------------------------------
    // Component registration (lazy – not required before insert)
    // ------------------------------------------------------------------

    pub fn register_component<T: 'static>(&mut self) {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentStore::<T>::new()));
    }

    // ------------------------------------------------------------------
    // Component CRUD
    // ------------------------------------------------------------------

    pub fn insert<T: 'static>(&mut self, entity: Entity, component: T) {
        self.register_component::<T>();
        self.get_storage_mut::<T>()
            .unwrap()
            .insert(entity.0, component);
    }

    pub fn remove<T: 'static>(&mut self, entity: Entity) {
        if let Some(storage) = self.get_storage_mut::<T>() {
            storage.remove(entity.0);
        }
    }

    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.get_storage::<T>()?.get(entity.0)
    }

    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.get_storage_mut::<T>()?.get_mut(entity.0)
    }

    pub fn has_component<T: 'static>(&self, entity_id: usize) -> bool {
        self.get_storage::<T>()
            .map(|s| s.get(entity_id).is_some())
            .unwrap_or(false)
    }

    // ------------------------------------------------------------------
    // Raw storage access
    // ------------------------------------------------------------------

    pub fn get_storage<T: 'static>(&self) -> Option<&ComponentStore<T>> {
        self.storages
            .get(&TypeId::of::<T>())?
            .downcast_ref::<ComponentStore<T>>()
    }

    pub fn get_storage_mut<T: 'static>(&mut self) -> Option<&mut ComponentStore<T>> {
        self.storages
            .get_mut(&TypeId::of::<T>())?
            .downcast_mut::<ComponentStore<T>>()
    }

    // ------------------------------------------------------------------
    // Single-component query (no filter)
    // ------------------------------------------------------------------

    /// Iterate all alive entities with component `A`.
    pub fn query<A: 'static>(&self) -> impl Iterator<Item = (Entity, &A)> {
        self.query_filtered::<A, NoFilter>(NoFilter)
    }

    /// Iterate alive entities with component `A`, applying `filter`.
    pub fn query_filtered<'a, A: 'static, F: QueryFilter + 'a>(
        &'a self,
        filter: F,
    ) -> impl Iterator<Item = (Entity, &'a A)> + 'a {
        let storage = self.get_storage::<A>();
        let max = self.entities_count;
        (0..max).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) {
                return None;
            }
            if !filter.matches(id, self) {
                return None;
            }
            let comp = storage?.get(id)?;
            Some((Entity(id), comp))
        })
    }

    // ------------------------------------------------------------------
    // Two-component tuple query
    // ------------------------------------------------------------------

    /// Iterate all alive entities that have **both** `A` and `B`.
    pub fn query2<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        self.query2_filtered::<A, B, NoFilter>(NoFilter)
    }

    pub fn query2_filtered<'a, A: 'static, B: 'static, F: QueryFilter + 'a>(
        &'a self,
        filter: F,
    ) -> impl Iterator<Item = (Entity, &'a A, &'a B)> + 'a {
        let sa = self.get_storage::<A>();
        let sb = self.get_storage::<B>();
        let max = self.entities_count;
        (0..max).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) {
                return None;
            }
            if !filter.matches(id, self) {
                return None;
            }
            let a = sa?.get(id)?;
            let b = sb?.get(id)?;
            Some((Entity(id), a, b))
        })
    }

    // ------------------------------------------------------------------
    // Three-component tuple query
    // ------------------------------------------------------------------

    pub fn query3<A: 'static, B: 'static, C: 'static>(
        &self,
    ) -> impl Iterator<Item = (Entity, &A, &B, &C)> {
        self.query3_filtered::<A, B, C, NoFilter>(NoFilter)
    }

    pub fn query3_filtered<'a, A: 'static, B: 'static, C: 'static, F: QueryFilter + 'a>(
        &'a self,
        filter: F,
    ) -> impl Iterator<Item = (Entity, &'a A, &'a B, &'a C)> + 'a {
        let sa = self.get_storage::<A>();
        let sb = self.get_storage::<B>();
        let sc = self.get_storage::<C>();
        let max = self.entities_count;
        (0..max).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) {
                return None;
            }
            if !filter.matches(id, self) {
                return None;
            }
            let a = sa?.get(id)?;
            let b = sb?.get(id)?;
            let c = sc?.get(id)?;
            Some((Entity(id), a, b, c))
        })
    }

    // ------------------------------------------------------------------
    // Optional-component query helper
    // ------------------------------------------------------------------
    //
    // Returns (Entity, &A, Option<&B>) — entities that have A, optionally B.
    //

    pub fn query_optional<'a, A: 'static, B: 'static>(
        &'a self,
    ) -> impl Iterator<Item = (Entity, &'a A, Option<&'a B>)> + 'a {
        let sa = self.get_storage::<A>();
        let sb = self.get_storage::<B>();
        let max = self.entities_count;
        (0..max).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) {
                return None;
            }
            let a = sa?.get(id)?;
            let b = sb.and_then(|s| s.get(id));
            Some((Entity(id), a, b))
        })
    }
}