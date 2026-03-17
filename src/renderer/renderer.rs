use sdl3::render::Canvas;
use sdl3::video::{Window};
//use sdl3::image::LoadTexture;
use sdl3::rect::Rect;

pub mod texture_cache;
use  texture_cache::TextureCache;

use crate::player::Player;

//use std::collections::HashMap;
//use std::path::Path;
pub struct Renderer<'a> {
    canvas: Canvas<Window>,
    // The Renderer doesn't OWN the textures; it just borrows the Manager
    // to draw what it needs.
    assets: &'a TextureCache<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(canvas: Canvas<Window>, assets: &'a TextureCache<'a>) -> Self {
        Self { canvas, assets }
    }
    pub fn draw_sprite(&mut self, x:i32, y:i32, w:u32, h:u32) {
        if let Some(tex) = self.assets.get("assets/test.png") {
            let dst = Rect::new(x, y, w, h);
            let _ = self.canvas.copy(tex, None, dst);
        }
    }
    pub fn draw_background(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(sdl3::pixels::Color::RGB(0, 100, 0));
        self.canvas.clear();
        Ok(())
    }
    pub fn present(&mut self) {
        self.canvas.present();
    }
    // pub fn draw(&mut self, player: &Option<Player>) -> Result<(), String> {
    //     self.canvas.set_draw_color(sdl3::pixels::Color::RGB(0, 100, 0));
    //     self.canvas.clear();


    //     // Borrow a texture from the manager and draw it
    //     if let Some(tex) = self.assets.get("assets/test.png") {
    //         if let Some(player) = player{
    //             let dst = Rect::new(player.position.x as i32, player.position.y as i32, 256, 256);
    //             let _ = self.canvas.copy(tex, None, dst);
    //         }
    //     }

    //     self.canvas.present();
    //     Ok(())
    // }
}