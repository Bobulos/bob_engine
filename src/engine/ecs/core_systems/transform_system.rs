use crate::entities::core_components::{Transform,Sprite};
use crate::entities::DynamicWorld;
use crate::rendering;

pub fn transform_system(world: &DynamicWorld, renderer: &mut rendering::Renderer) {
    // If BOTH storages exist, proceed to the block
    if let (Some(sprites), Some(transforms)) = (
        world.get_storage::<Sprite>(), 
        world.get_storage::<Transform>()
    ) {
        for (id, sprite) in sprites.iter() {
            if let Some(pos) = transforms.get(id) {
                // draw logic here
                //renderer.draw_sprite(pos.position.x as i32, pos.position.y as i32, sprite.width, sprite.height);
            }
        }
    }
}