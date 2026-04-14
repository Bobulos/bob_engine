use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::component_store::ComponentStore;
use crate::b_engine::entities::query::{QueryFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

pub struct DynamicWorld {
    storages:       RwLock<HashMap<TypeId, Arc<RwLock<Box<dyn Any + Send + Sync>>>>>,
    alive:          RwLock<Vec<bool>>,
    entities_count: RwLock<usize>,
}

impl DynamicWorld {
    pub fn new() -> Self {
        Self {
            storages:       RwLock::new(HashMap::new()),
            alive:          RwLock::new(Vec::new()),
            entities_count: RwLock::new(0),
        }
    }

    // ── Internal storage helpers ──────────────────────────────────────────────

    fn storage_arc<T: Any + Send + Sync + 'static>(
        &self,
    ) -> Option<Arc<RwLock<Box<dyn Any + Send + Sync>>>> {
        self.storages.read().unwrap().get(&TypeId::of::<T>()).cloned()
    }

    fn with_storage<T, R>(&self, f: impl FnOnce(&ComponentStore<T>) -> R) -> Option<R>
    where
        T: Any + Send + Sync + 'static,
    {
        let arc = self.storage_arc::<T>()?;
        let guard = arc.read().unwrap();
        let store = guard.downcast_ref::<ComponentStore<T>>()?;
        Some(f(store))
    }

    fn with_storage_mut<T, R>(&self, f: impl FnOnce(&mut ComponentStore<T>) -> R) -> Option<R>
    where
        T: Any + Send + Sync + 'static,
    {
        let arc = self.storage_arc::<T>()?;
        let mut guard = arc.write().unwrap();
        let store = guard.downcast_mut::<ComponentStore<T>>()?;
        Some(f(store))
    }

    // ── Entity management ─────────────────────────────────────────────────────

    pub fn spawn(&self) -> Entity {
        let mut count = self.entities_count.write().unwrap();
        let id = *count;
        *count += 1;
        let mut alive = self.alive.write().unwrap();
        if id >= alive.len() {
            alive.resize(id + 1, false);
        }
        alive[id] = true;
        Entity(id)
    }

    pub fn despawn(&self, entity: Entity) {
        if let Some(slot) = self.alive.write().unwrap().get_mut(entity.0) {
            *slot = false;
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive.read().unwrap().get(entity.0).copied().unwrap_or(false)
    }

    pub fn entity_count(&self) -> usize {
        self.alive.read().unwrap().iter().filter(|&&a| a).count()
    }

    // ── Component CRUD ────────────────────────────────────────────────────────

    pub fn register_component<T: Any + Send + Sync + 'static>(&self) {
        self.storages
            .write()
            .unwrap()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Arc::new(RwLock::new(Box::new(ComponentStore::<T>::new()))));
    }

    pub fn insert<T: Any + Send + Sync + 'static>(&self, entity: Entity, component: T) {
        self.register_component::<T>();
        self.with_storage_mut::<T, _>(|s| s.insert(entity.0, component));
    }

    pub fn remove<T: Any + Send + Sync + 'static>(&self, entity: Entity) {
        self.with_storage_mut::<T, _>(|s| s.remove(entity.0));
    }

    pub fn get_clone<T: Any + Send + Sync + Clone + 'static>(&self, entity: Entity) -> Option<T> {
        self.with_storage::<T, _>(|s| s.get(entity.0).cloned())?
    }

    pub fn has_component<T: Any + Send + Sync + 'static>(&self, entity_id: usize) -> bool {
        self.with_storage::<T, _>(|s| s.get(entity_id).is_some())
            .unwrap_or(false)
    }

    // ── Closure-based queries (immutable) ─────────────────────────────────────

    pub fn for_each<A>(&self, mut f: impl FnMut(Entity, &A))
    where
        A: Any + Send + Sync + 'static,
    {
        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();
        self.with_storage::<A, _>(|sa| {
            for id in 0..count {
                if !alive.get(id).copied().unwrap_or(false) { continue; }
                if let Some(a) = sa.get(id) {
                    f(Entity(id), a);
                }
            }
        });
    }

    pub fn for_each2<A, B>(&self, mut f: impl FnMut(Entity, &A, &B))
    where
        A: Any + Send + Sync + 'static,
        B: Any + Send + Sync + 'static,
    {
        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let arc_b = match self.storage_arc::<B>() { Some(b) => b, None => return };
        let guard_a = arc_a.read().unwrap();
        let guard_b = arc_b.read().unwrap();
        let sa = guard_a.downcast_ref::<ComponentStore<A>>().unwrap();
        let sb = guard_b.downcast_ref::<ComponentStore<B>>().unwrap();

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if let (Some(a), Some(b)) = (sa.get(id), sb.get(id)) {
                f(Entity(id), a, b);
            }
        }
    }

    pub fn for_each3<A, B, C>(&self, mut f: impl FnMut(Entity, &A, &B, &C))
    where
        A: Any + Send + Sync + 'static,
        B: Any + Send + Sync + 'static,
        C: Any + Send + Sync + 'static,
    {
        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let arc_b = match self.storage_arc::<B>() { Some(b) => b, None => return };
        let arc_c = match self.storage_arc::<C>() { Some(c) => c, None => return };
        let guard_a = arc_a.read().unwrap();
        let guard_b = arc_b.read().unwrap();
        let guard_c = arc_c.read().unwrap();
        let sa = guard_a.downcast_ref::<ComponentStore<A>>().unwrap();
        let sb = guard_b.downcast_ref::<ComponentStore<B>>().unwrap();
        let sc = guard_c.downcast_ref::<ComponentStore<C>>().unwrap();

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if let (Some(a), Some(b), Some(c)) = (sa.get(id), sb.get(id), sc.get(id)) {
                f(Entity(id), a, b, c);
            }
        }
    }

    pub fn for_each_optional<A, B>(&self, mut f: impl FnMut(Entity, &A, Option<&B>))
    where
        A: Any + Send + Sync + 'static,
        B: Any + Send + Sync + 'static,
    {
        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let guard_a = arc_a.read().unwrap();
        let sa = guard_a.downcast_ref::<ComponentStore<A>>().unwrap();

        // B is optional — don't bail if missing, just pass None
        let arc_b = self.storage_arc::<B>();
        let guard_b = arc_b.as_ref().map(|a| a.read().unwrap());
        let sb = guard_b.as_ref().and_then(|g| g.downcast_ref::<ComponentStore<B>>());

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if let Some(a) = sa.get(id) {
                f(Entity(id), a, sb.and_then(|s| s.get(id)));
            }
        }
    }

    // ── Closure-based queries (mutable) ───────────────────────────────────────

    pub fn for_each_mut<A>(&self, mut f: impl FnMut(Entity, &mut A))
    where
        A: Any + Send + Sync + 'static,
    {
        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let mut guard_a = arc_a.write().unwrap();
        let sa = guard_a.downcast_mut::<ComponentStore<A>>().unwrap();

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if let Some(a) = sa.get_mut(id) {
                f(Entity(id), a);
            }
        }
    }

    /// A mutable, B immutable.
    pub fn for_each2_mut<A, B>(&self, mut f: impl FnMut(Entity, &mut A, &B))
    where
        A: Any + Send + Sync + 'static,
        B: Any + Send + Sync + 'static,
    {
        assert_ne!(TypeId::of::<A>(), TypeId::of::<B>(), "A and B must be different types");

        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let arc_b = match self.storage_arc::<B>() { Some(b) => b, None => return };
        let mut guard_a = arc_a.write().unwrap();
        let guard_b = arc_b.read().unwrap();
        let sa = guard_a.downcast_mut::<ComponentStore<A>>().unwrap();
        let sb = guard_b.downcast_ref::<ComponentStore<B>>().unwrap();

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if let (Some(a), Some(b)) = (sa.get_mut(id), sb.get(id)) {
                f(Entity(id), a, b);
            }
        }
    }

    /// A and B both mutable.
    pub fn for_each2_mut_both<A, B>(&self, mut f: impl FnMut(Entity, &mut A, &mut B))
    where
        A: Any + Send + Sync + 'static,
        B: Any + Send + Sync + 'static,
    {
        assert_ne!(TypeId::of::<A>(), TypeId::of::<B>(), "A and B must be different types");

        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let arc_b = match self.storage_arc::<B>() { Some(b) => b, None => return };
        let mut guard_a = arc_a.write().unwrap();
        let mut guard_b = arc_b.write().unwrap();
        let sa = guard_a.downcast_mut::<ComponentStore<A>>().unwrap();
        let sb = guard_b.downcast_mut::<ComponentStore<B>>().unwrap();

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if let (Some(a), Some(b)) = (sa.get_mut(id), sb.get_mut(id)) {
                f(Entity(id), a, b);
            }
        }
    }

    /// A and B mutable, C immutable.
    pub fn for_each3_mut<A, B, C>(&self, mut f: impl FnMut(Entity, &mut A, &mut B, &C))
    where
        A: Any + Send + Sync + 'static,
        B: Any + Send + Sync + 'static,
        C: Any + Send + Sync + 'static,
    {
        assert_ne!(TypeId::of::<A>(), TypeId::of::<B>(), "A and B must be different types");
        assert_ne!(TypeId::of::<A>(), TypeId::of::<C>(), "A and C must be different types");
        assert_ne!(TypeId::of::<B>(), TypeId::of::<C>(), "B and C must be different types");

        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let arc_b = match self.storage_arc::<B>() { Some(b) => b, None => return };
        let arc_c = match self.storage_arc::<C>() { Some(c) => c, None => return };
        let mut guard_a = arc_a.write().unwrap();
        let mut guard_b = arc_b.write().unwrap();
        let guard_c = arc_c.read().unwrap();
        let sa = guard_a.downcast_mut::<ComponentStore<A>>().unwrap();
        let sb = guard_b.downcast_mut::<ComponentStore<B>>().unwrap();
        let sc = guard_c.downcast_ref::<ComponentStore<C>>().unwrap();

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if let (Some(a), Some(b), Some(c)) = (sa.get_mut(id), sb.get_mut(id), sc.get(id)) {
                f(Entity(id), a, b, c);
            }
        }
    }

    // ── Filtered variants ─────────────────────────────────────────────────────

    pub fn for_each_filtered<A, F>(&self, filter: F, mut f: impl FnMut(Entity, &A))
    where
        A: Any + Send + Sync + 'static,
        F: QueryFilter,
    {
        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();
        self.with_storage::<A, _>(|sa| {
            for id in 0..count {
                if !alive.get(id).copied().unwrap_or(false) { continue; }
                if !filter.matches(id, self) { continue; }
                if let Some(a) = sa.get(id) {
                    f(Entity(id), a);
                }
            }
        });
    }

    pub fn for_each_mut_filtered<A, F>(&self, filter: F, mut f: impl FnMut(Entity, &mut A))
    where
        A: Any + Send + Sync + 'static,
        F: QueryFilter,
    {
        let count = *self.entities_count.read().unwrap();
        let alive = self.alive.read().unwrap();

        let arc_a = match self.storage_arc::<A>() { Some(a) => a, None => return };
        let mut guard_a = arc_a.write().unwrap();
        let sa = guard_a.downcast_mut::<ComponentStore<A>>().unwrap();

        for id in 0..count {
            if !alive.get(id).copied().unwrap_or(false) { continue; }
            if !filter.matches(id, self) { continue; }
            if let Some(a) = sa.get_mut(id) {
                f(Entity(id), a);
            }
        }
    }
}