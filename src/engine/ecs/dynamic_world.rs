use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::component_store::ComponentStore;
use crate::entities::query::{NoFilter, QueryFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

pub struct DynamicWorld {
    storages:       HashMap<TypeId, Box<dyn Any>>,
    alive:          Vec<bool>,
    entities_count: usize,
}

impl DynamicWorld {
    pub fn new() -> Self {
        Self {
            storages:       HashMap::new(),
            alive:          Vec::new(),
            entities_count: 0,
        }
    }

    // ── Entity management ─────────────────────────────────────────────────────

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

    // ── Component CRUD ────────────────────────────────────────────────────────

    pub fn register_component<T: 'static>(&mut self) {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentStore::<T>::new()));
    }

    pub fn insert<T: 'static>(&mut self, entity: Entity, component: T) {
        self.register_component::<T>();
        self.get_storage_mut::<T>().unwrap().insert(entity.0, component);
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

    // ── Raw storage access ────────────────────────────────────────────────────

    pub fn get_storage<T: 'static>(&self) -> Option<&ComponentStore<T>> {
        self.storages.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn get_storage_mut<T: 'static>(&mut self) -> Option<&mut ComponentStore<T>> {
        self.storages.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    // ── Immutable queries (borrow, iterate, read) ─────────────────────────────

    pub fn query<A: 'static>(&self) -> impl Iterator<Item = (Entity, &A)> {
        self.query_filtered::<A, NoFilter>(NoFilter)
    }

    pub fn query_filtered<'a, A: 'static, F: QueryFilter + 'a>(
        &'a self,
        filter: F,
    ) -> impl Iterator<Item = (Entity, &'a A)> + 'a {
        let storage = self.get_storage::<A>();
        (0..self.entities_count).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) { return None; }
            if !filter.matches(id, self) { return None; }
            Some((Entity(id), storage?.get(id)?))
        })
    }

    pub fn query2<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        self.query2_filtered::<A, B, NoFilter>(NoFilter)
    }

    pub fn query2_filtered<'a, A: 'static, B: 'static, F: QueryFilter + 'a>(
        &'a self,
        filter: F,
    ) -> impl Iterator<Item = (Entity, &'a A, &'a B)> + 'a {
        let sa = self.get_storage::<A>();
        let sb = self.get_storage::<B>();
        (0..self.entities_count).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) { return None; }
            if !filter.matches(id, self) { return None; }
            Some((Entity(id), sa?.get(id)?, sb?.get(id)?))
        })
    }

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
        (0..self.entities_count).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) { return None; }
            if !filter.matches(id, self) { return None; }
            Some((Entity(id), sa?.get(id)?, sb?.get(id)?, sc?.get(id)?))
        })
    }

    pub fn query_optional<'a, A: 'static, B: 'static>(
        &'a self,
    ) -> impl Iterator<Item = (Entity, &'a A, Option<&'a B>)> + 'a {
        let sa = self.get_storage::<A>();
        let sb = self.get_storage::<B>();
        (0..self.entities_count).filter_map(move |id| {
            if !self.alive.get(id).copied().unwrap_or(false) { return None; }
            Some((Entity(id), sa?.get(id)?, sb.and_then(|s| s.get(id))))
        })
    }

    // ── Mutable queries ───────────────────────────────────────────────────────
    //
    // These collect matching entity IDs first, then hand back &mut references
    // one at a time. Collecting IDs breaks the borrow so mutation is safe.
    //

    /// Iterate all alive entities with `A`, mutably.
    pub fn query_mut<A: 'static>(&mut self) -> impl Iterator<Item = (Entity, &mut A)> {
        let ids = self.matching_ids::<A>();
        let storage = self.get_storage_mut::<A>().unwrap() as *mut ComponentStore<A>;

        ids.into_iter().filter_map(move |id| {
            // SAFETY: ids are unique so each get_mut touches a distinct slot,
            // and storage outlives this iterator since it lives on self.
            let comp = unsafe { (*storage).get_mut(id)? };
            Some((Entity(id), comp))
        })
    }

    /// Iterate alive entities with both `A` and `B`, mutably on `A`.
    /// `B` is immutable — Rust can't safely give two &mut to different
    /// components without unsafe or a split-borrow helper (see query2_mut_both).
    pub fn query2_mut<A: 'static, B: 'static>(
        &mut self,
    ) -> impl Iterator<Item = (Entity, &mut A, &B)> {
        let ids = self.matching_ids2::<A, B>();

        // Split storages so we can hold &mut A and &B simultaneously
        let sa = self.storages.get_mut(&TypeId::of::<A>()).unwrap()
            .downcast_mut::<ComponentStore<A>>().unwrap() as *mut ComponentStore<A>;
        let sb = self.storages.get(&TypeId::of::<B>()).unwrap()
            .downcast_ref::<ComponentStore<B>>().unwrap() as *const ComponentStore<B>;

        ids.into_iter().filter_map(move |id| {
            // SAFETY: sa and sb are different TypeId keys — distinct allocations.
            // ids are unique so each sa.get_mut(id) is a non-overlapping slot.
            let a = unsafe { (*sa).get_mut(id)? };
            let b = unsafe { (*sb).get(id)? };
            Some((Entity(id), a, b))
        })
    }

    /// Mutable access to both `A` and `B` on matched entities.
    pub fn query2_mut_both<A: 'static, B: 'static>(
        &mut self,
    ) -> impl Iterator<Item = (Entity, &mut A, &mut B)> {
        assert_ne!(
            TypeId::of::<A>(), TypeId::of::<B>(),
            "query2_mut_both: A and B must be different types"
        );
        let ids = self.matching_ids2::<A, B>();

        let sa = self.storages.get_mut(&TypeId::of::<A>()).unwrap()
            .downcast_mut::<ComponentStore<A>>().unwrap() as *mut ComponentStore<A>;
        let sb = self.storages.get_mut(&TypeId::of::<B>()).unwrap()
            .downcast_mut::<ComponentStore<B>>().unwrap() as *mut ComponentStore<B>;

        ids.into_iter().filter_map(move |id| {
            // SAFETY: A != B enforced above, so sa and sb are non-aliasing.
            // ids are unique so each get_mut call touches a distinct slot.
            let a = unsafe { (*sa).get_mut(id)? };
            let b = unsafe { (*sb).get_mut(id)? };
            Some((Entity(id), a, b))
        })
    }

    /// Three-component mutable query — A and B mutable, C immutable.
    pub fn query3_mut<A: 'static, B: 'static, C: 'static>(
        &mut self,
    ) -> impl Iterator<Item = (Entity, &mut A, &mut B, &C)> {
        assert_ne!(TypeId::of::<A>(), TypeId::of::<B>(),
            "query3_mut: A and B must be different types");

        let ids = self.matching_ids3::<A, B, C>();

        let sa = self.storages.get_mut(&TypeId::of::<A>()).unwrap()
            .downcast_mut::<ComponentStore<A>>().unwrap() as *mut ComponentStore<A>;
        let sb = self.storages.get_mut(&TypeId::of::<B>()).unwrap()
            .downcast_mut::<ComponentStore<B>>().unwrap() as *mut ComponentStore<B>;
        let sc = self.storages.get(&TypeId::of::<C>()).unwrap()
            .downcast_ref::<ComponentStore<C>>().unwrap() as *const ComponentStore<C>;

        ids.into_iter().filter_map(move |id| {
            let a = unsafe { (*sa).get_mut(id)? };
            let b = unsafe { (*sb).get_mut(id)? };
            let c = unsafe { (*sc).get(id)? };
            Some((Entity(id), a, b, c))
        })
    }

    // ── Collect-IDs helpers (used internally by mutable queries) ──────────────

    fn matching_ids<A: 'static>(&self) -> Vec<usize> {
        let storage = self.get_storage::<A>();
        (0..self.entities_count)
            .filter(|&id| {
                self.alive.get(id).copied().unwrap_or(false)
                    && storage.map(|s| s.get(id).is_some()).unwrap_or(false)
            })
            .collect()
    }

    fn matching_ids2<A: 'static, B: 'static>(&self) -> Vec<usize> {
        let sa = self.get_storage::<A>();
        let sb = self.get_storage::<B>();
        (0..self.entities_count)
            .filter(|&id| {
                self.alive.get(id).copied().unwrap_or(false)
                    && sa.map(|s| s.get(id).is_some()).unwrap_or(false)
                    && sb.map(|s| s.get(id).is_some()).unwrap_or(false)
            })
            .collect()
    }

    fn matching_ids3<A: 'static, B: 'static, C: 'static>(&self) -> Vec<usize> {
        let sa = self.get_storage::<A>();
        let sb = self.get_storage::<B>();
        let sc = self.get_storage::<C>();
        (0..self.entities_count)
            .filter(|&id| {
                self.alive.get(id).copied().unwrap_or(false)
                    && sa.map(|s| s.get(id).is_some()).unwrap_or(false)
                    && sb.map(|s| s.get(id).is_some()).unwrap_or(false)
                    && sc.map(|s| s.get(id).is_some()).unwrap_or(false)
            })
            .collect()
    }
}