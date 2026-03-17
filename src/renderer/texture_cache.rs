use sdl3::render::{Texture, TextureCreator};
use sdl3::video::{WindowContext};
use sdl3::image::LoadTexture;
use std::collections::HashMap;
use std::path::Path;


// --- 1. OWNERSHIP: THE TEXTURE MANAGER ---
// This struct "owns" the textures. It uses a lifetime 'a to say
// "These textures will live as long as the TextureCreator exists."
pub struct TextureCache<'a> {
    loader: &'a TextureCreator<WindowContext>,
    cache: HashMap<String, Texture<'a>>,
}

impl<'a> TextureCache<'a> {
    pub fn new(loader: &'a TextureCreator<WindowContext>) -> Self {
        Self { loader, cache: HashMap::new() }
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let texture = self.loader.load_texture(Path::new(path))
            .map_err(|e| e.to_string())?;
        self.cache.insert(path.to_string(), texture);
        Ok(())
    }

    pub fn get(&self, path: &str) -> Option<&Texture<'a>> {
        self.cache.get(path)
    }
}

