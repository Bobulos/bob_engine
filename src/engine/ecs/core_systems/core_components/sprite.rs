#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub texture_id: u32,       // ID to look up in your TextureCache
    pub width: u32,
    pub height: u32,
    //pub source_rect: Option<sdl3::rect::Rect>, // Optional: for spritesheets/animation
}

impl Sprite {
    pub fn new(texture_id: u32, width: u32, height: u32) -> Self {
        Self {
            texture_id,
            width,
            height,
            //source_rect: None,
        }
    }
}