use crate::vec::Float2;

pub struct Player
{
    pub position: Float2,
    pub sprite_id: i32,
}
impl Player {
    pub fn new() -> Self {
        Self { position: Float2 { x: 0f32, y: 0f32}, sprite_id: (100i32) }
        
    }
    pub fn move_player(&mut self){
        self.position.x += 0.01f32;
        self.position.y += 0.01f32;
    }
}