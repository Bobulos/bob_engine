pub struct Camera {
    pub position: [f32; 2],  // world position the camera is looking at
    pub zoom: f32,           // 1.0 = normal, 2.0 = zoomed in
    pub viewport_width: u32,
    pub viewport_height: u32,
}

impl Camera {

    pub fn move_by(&mut self, dx: f32, dy: f32) {
        self.position[0] += dx;
        self.position[1] += dy;
    }

    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.001, 1.0);
    }

    pub fn zoom_towards(&mut self, factor: f32, world_x: f32, world_y: f32) {
        // Zoom keeping a specific world point fixed on screen (e.g. mouse position)
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.001, 1.0);
        let zoom_change = self.zoom / old_zoom;
        self.position[0] = world_x + (self.position[0] - world_x) / zoom_change;
        self.position[1] = world_y + (self.position[1] - world_y) / zoom_change;
    }
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            position: [0.0, 0.0],
            zoom: 1.0,
            viewport_width: 1,
            viewport_height: 1,
        }
    }
    // Produces an orthographic projection that preserves aspect ratio
    pub fn build_matrix(&self) -> [[f32; 4]; 4] {
        let aspect = self.viewport_width as f32 / self.viewport_height as f32;

        // World units visible on each axis at zoom 1.0
        // e.g. at zoom 1.0, you see 2.0 world units tall, aspect * 2.0 wide
        let half_h = 1.0 / self.zoom;
        let half_w = half_h * aspect;

        let l = self.position[0] - half_w;
        let r = self.position[0] + half_w;
        let b = self.position[1] - half_h;
        let t = self.position[1] + half_h;

        // Column-major orthographic projection matrix
        // Maps [l,r] x [b,t] x [0,1] into clip space [-1,1] x [-1,1] x [0,1]
        [
            [2.0 / (r - l),       0.0,              0.0, 0.0],
            [0.0,                 2.0 / (t - b),    0.0, 0.0],
            [0.0,                 0.0,               1.0, 0.0],
            [-(r+l)/(r-l),  -(t+b)/(t-b),      0.0, 1.0],
        ]
    }
}