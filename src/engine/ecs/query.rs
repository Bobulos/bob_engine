/// A query filter that restricts which entities are returned.
///
/// Implement this to add arbitrary per entity conditions on top of a tuple query.
/// Common built-in filters: `With<T>`, `Without<T>`, `Changed<T>` (see below).

use crate::b_engine::entities::dynamic_world::DynamicWorld;
use std::marker::PhantomData;
use std::any::Any;
pub trait QueryFilter: Send + Sync {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool;
}

// With<T>
pub struct With<T: Any + Send + Sync + 'static>(PhantomData<T>);

impl<T: Any + Send + Sync + 'static> With<T> {
    pub fn new() -> Self { Self(PhantomData) }
}

impl<T: Any + Send + Sync + 'static> QueryFilter for With<T> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        world.has_component::<T>(entity_id)
    }
}

// Without<T>
pub struct Without<T: Any + Send + Sync + 'static>(PhantomData<T>);

impl<T: Any + Send + Sync + 'static> Without<T> {
    pub fn new() -> Self { Self(PhantomData) }
}

impl<T: Any + Send + Sync + 'static> QueryFilter for Without<T> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        !world.has_component::<T>(entity_id)
    }
}

// And / Or / NoFilter don't hold T directly so they just need
// their inner filters to be Send + Sync, which the trait bound covers
pub struct And<A: QueryFilter, B: QueryFilter>(pub A, pub B);
impl<A: QueryFilter, B: QueryFilter> QueryFilter for And<A, B> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        self.0.matches(entity_id, world) && self.1.matches(entity_id, world)
    }
}

pub struct Or<A: QueryFilter, B: QueryFilter>(pub A, pub B);
impl<A: QueryFilter, B: QueryFilter> QueryFilter for Or<A, B> {
    fn matches(&self, entity_id: usize, world: &DynamicWorld) -> bool {
        self.0.matches(entity_id, world) || self.1.matches(entity_id, world)
    }
}

pub struct NoFilter;
impl QueryFilter for NoFilter {
    fn matches(&self, _entity_id: usize, _world: &DynamicWorld) -> bool { true }
}