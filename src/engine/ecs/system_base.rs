use std::sync::Arc;
use crate::b_engine::entities::DynamicWorld;

pub trait SystemBase: Send + Sync {
    fn on_start(&self, world: &Arc<DynamicWorld>);
    fn on_update(&self, world: &Arc<DynamicWorld>);
    fn on_destroy(&self, world: &Arc<DynamicWorld>);
}