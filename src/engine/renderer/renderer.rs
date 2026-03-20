use sdl3::render::Canvas;
use sdl3::video::Window;
use sdl3::rect::Rect;
use sdl3::render::Texture;
use std::ffi::CString;
use crate::rendering::camera::Camera;
use crate::coords::Float2;

use sdl3::sys::gpu::*;
pub struct Renderer {
    window: Window,
//    canvas: Canvas<Window>,
    // assets: &'a TextureCache<'a>,
    // test_texture: Option<&'a Texture<'a>>,
    camera: Camera,
}

impl Renderer {
    pub fn new(window:Window
    //    , assets: &'a TextureCache<'a>
    ) -> Self {
        Self {
            window,
            // assets,
            // test_texture: None,
            camera: Camera::new(1f32,1080u32, 720u32, 32)
        }
    }

    pub fn init(&mut self) {
        let backend = CString::new("vulkan").expect("CString failed");

        let device = unsafe {
            SDL_CreateGPUDevice(SDL_GPU_SHADERFORMAT_SPIRV, true, backend.as_ptr())
        };

        if device.is_null() {
            panic!("Failed to create GPU device");
        }

        unsafe {
            if !SDL_ClaimWindowForGPUDevice(device, self.window.raw()) {
                panic!("Failed to claim window for GPU device");
            }
        }

        //self.test_texture = self.assets.get("assets/test.png");
        self.camera.snap_to(Float2 { x: 200f32, y: 300f32 });
    }
    
    // Deprecated use gpu hardware accelerate
    // pub fn draw_sprite(&mut self, position: Float2, world_bounds: f32) {
    //     if let Some(tex) = self.test_texture {
    //         // let screen_pos = self.camera.world_to_screen(Float2::new(position.x, position.y));
    //         // let dst = Rect::new(screen_pos.x as i32, screen_pos.y as i32, 32, 32);
    //         let dst_sprite= self.camera.world_to_screen_rect(position, world_bounds);
    //         let _ = self.canvas.copy(tex, None, dst_sprite);
    //     }
    // }

    // pub fn draw_background(&mut self) -> Result<(), String> {
    //     self.canvas.set_draw_color(sdl3::pixels::Color::RGB(0, 100, 0));
    //     self.canvas.clear();
    //     Ok(())
    // }

    // pub fn present(&mut self) {
    //     self.camera.follow(Float2 { x: 0f32, y: 0f32 }, 1f32, 1f32/60f32);
    //     self.canvas.present();
    // }
}