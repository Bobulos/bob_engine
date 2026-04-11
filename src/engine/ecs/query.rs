/// A query filter that restricts which entities are returned.
///
/// Implement this to add arbitrary per entity conditions on top of a tuple query.
/// Common built-in filters: `With<T>`, `Without<T>`, `Changed<T>` (see below).

use crate::b_engine::entities::dynamic_world::DynamicWorld;
pub trait QueryFilter {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool;
}

// ---------------------------------------------------------------------------
// With<T>  –  entity must have component T
// ---------------------------------------------------------------------------
pub struct With<T: 'static>(std::marker::PhantomData<T>);

impl<T: 'static> With<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: 'static> QueryFilter for With<T> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        world.has_component::<T>(entity_id)
    }
}

// ---------------------------------------------------------------------------
// Without<T>  –  entity must NOT have component T
// ---------------------------------------------------------------------------
pub struct Without<T: 'static>(std::marker::PhantomData<T>);

impl<T: 'static> Without<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: 'static> QueryFilter for Without<T> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        !world.has_component::<T>(entity_id)
    }
}

// ---------------------------------------------------------------------------
// And<A, B>  –  combine two filters with logical AND
// ---------------------------------------------------------------------------
pub struct And<A: QueryFilter, B: QueryFilter>(pub A, pub B);

impl<A: QueryFilter, B: QueryFilter> QueryFilter for And<A, B> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        self.0.matches(entity_id, world) && self.1.matches(entity_id, world)
    }
}

// ---------------------------------------------------------------------------
// Or<A, B>  –  combine two filters with logical OR
// ---------------------------------------------------------------------------
pub struct Or<A: QueryFilter, B: QueryFilter>(pub A, pub B);

impl<A: QueryFilter, B: QueryFilter> QueryFilter for Or<A, B> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        self.0.matches(entity_id, world) || self.1.matches(entity_id, world)
    }
}

// ---------------------------------------------------------------------------
// NoFilter  –  pass-through, always true
// ---------------------------------------------------------------------------
pub struct NoFilter;

impl QueryFilter for NoFilter {
    fn matches(&self, _entity_id: usize, _world: &DynamicWorld) -> bool {
        true
    }
}