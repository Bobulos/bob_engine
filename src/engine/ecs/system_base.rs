use std::sync::Arc;
use crate::b_engine::entities::DynamicWorld;

pub trait SystemBase: Send + Sync {
    fn on_start(&mut self, world: &Arc<DynamicWorld>);
    fn on_update(&mut self, world: &Arc<DynamicWorld>);
    fn on_destroy(&mut self, world: &Arc<DynamicWorld>);
}