use crate::coords::{Float2, Int2};

pub struct Camera {
    pub position: Float2,       // world coords, float for smooth movement
    pub viewport_w: u32,
    pub viewport_h: u32,

    viewport_half_w: u32,
    viewport_half_h: u32,

    //This is zoom
    pub view_size_world: f32,

    pub pixels_per_unit: u32,

    adjusted_pixels_per_unit: f32
    // map_px_w: u32,
    // map_px_h: u32,
}

impl Camera {
    pub fn new(view_size_world: f32,viewport_w: u32, viewport_h: u32, pixels_per_unit: u32) -> Self {
        Self {
            view_size_world: view_size_world,
            position: Float2::ZERO,
            viewport_w,
            viewport_h,
            viewport_half_w: viewport_w/2,
            viewport_half_h: viewport_h/2,
            pixels_per_unit: pixels_per_unit,
            adjusted_pixels_per_unit: pixels_per_unit as f32/view_size_world
        }
    }
    // pub fn initialize(&mut self) {
    //     self.change_view_size(new_size);
    // }
    /// Move camera by a delta, clamped to map bounds
    pub fn move_by(&mut self, delta: Float2) {
        self.position = self.position + delta;
    }

    /// Smoothly follow a target position (call each frame with your delta time)
    pub fn follow(&mut self, target: Float2, speed: f32, dt: f32) {
        let t = (speed * dt).clamp(0.0, 1.0);
        self.position = self.position.lerp(target, t);
    }

    /// Snap directly to a position
    pub fn snap_to(&mut self, pos: Float2) {
        self.position = pos;
    }

    /// Convert world position to screen position
    pub fn world_to_screen(&self, world: Float2) -> Float2 {
        world - self.position
    }

    /// Convert screen position to world position
    pub fn screen_to_world(&self, screen: Float2) -> Float2 {
        screen + self.position
    }
    pub fn change_view_size(&mut self, new_size: f32)
    {
        self.view_size_world = new_size;

        // Matches to window height
        self.adjusted_pixels_per_unit = self.pixels_per_unit as f32/(self.view_size_world*self.viewport_h as f32);
    }
    /// Given world position and bounds returns the render rect only supports a uniform scale
    pub fn world_to_screen_rect(&self, world_position: Float2, world_bounds: f32) -> sdl3::rect::Rect {
        let screen_size = self.adjusted_pixels_per_unit * world_bounds;
        let half_size = screen_size / 2.0;

        let relative = world_position - self.position;
        let screen_px = relative * self.adjusted_pixels_per_unit;

        let screen_x = self.viewport_half_w as f32 + screen_px.x - half_size;
        let screen_y = self.viewport_half_h as f32 - screen_px.y - half_size;

        sdl3::rect::Rect::new(
            screen_x as i32,
            screen_y as i32,
            screen_size as u32,
            screen_size as u32,
        )
    }
    /// SDL src rect for blitting the map texture — truncates float to integer pixels
    pub fn src_rect(&self) -> sdl3::rect::Rect {
        sdl3::rect::Rect::new(
            self.position.x as i32,
            self.position.y as i32,
            self.viewport_w,
            self.viewport_h,
        )
    }

    // fn clamp(&self, pos: Float2) -> Float2 {
    //     Float2::new(
    //         pos.x.clamp(0.0, (self.map_px_w - self.viewport_w) as f32),
    //         pos.y.clamp(0.0, (self.map_px_h - self.viewport_h) as f32),
    //     )
    // }
}