use crate::rendering::Renderer;
use crate::input::Input;
use crate::entities::DynamicWorld;
/// Engine borrow references
pub struct EngineContext<'a> {
    pub world: &'a mut DynamicWorld,
    pub renderer: &'a mut Renderer,
    pub input:    &'a Input,
    pub delta:    f32,
}

pub trait System {
    fn on_start(&mut self, _ctx: &mut EngineContext) {}
    fn on_update(&mut self, ctx: &mut EngineContext);
    fn on_destroy(&mut self, _ctx: &mut EngineContext) {}
}