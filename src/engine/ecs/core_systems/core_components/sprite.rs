#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub intra_batch_index: usize, // Index into Renderer.batches
    pub batch_index: usize,       // Index into Renderer.batches
    //pub enabled: bool,          // Whether this sprite should be rendered
    pub texture_id: u32,       // ID to look up in your TextureCache
    pub width: u32,
    pub height: u32,
    //pub source_rect: Option<sdl3::rect::Rect>, // Optional: for spritesheets/animation
}

impl Sprite {
    pub fn new(texture_id: u32, width: u32, height: u32) -> Self {
        Self {
            intra_batch_index: 0, // Will be set later when the sprite is added to a batch
            batch_index: 0,
            //enabled: true,
            texture_id,
            width,
            height,
            //source_rect: None,
        }
    }
}