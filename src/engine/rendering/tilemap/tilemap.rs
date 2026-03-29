#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tile {
    pub tile_id: u32, // index into the tile atlas lookup
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TilemapInfo {
    pub width:      u32,
    pub height:     u32,
    pub atlas_rows: u32,
    pub atlas_cols: u32,
    pub offset:     [f32; 2], // world position of the tilemap origin
    pub _pad:       [f32; 2], // keep 16-byte alignment
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TileUV {
    pub offset: [f32; 2],
    pub scale: [f32; 2],
}

impl TileUV {
    /// Generates UVs for a grid-based texture atlas
    pub fn generate_atlas(cols: u32, rows: u32) -> Vec<Self> {
        let mut uvs = Vec::new();
        let u_scale = 1.0 / cols as f32;
        let v_scale = 1.0 / rows as f32;

        for y in 0..rows {
            for x in 0..cols {
                uvs.push(Self {
                    offset: [x as f32 * u_scale, y as f32 * v_scale],
                    scale: [u_scale, v_scale],
                });
            }
        }
        uvs
    }
}
pub struct Tilemap {
    pub tiles: Vec<Tile>,
    pub width: u32,   // in tiles
    pub height: u32,  // in tiles
    pub tile_size: f32, // world units per tile
    pub dirty: bool,  // only upload when true
}

impl Tilemap {
    pub fn new(width: u32, height: u32, tile_size: f32) -> Self {
        Self {
            tiles: vec![Tile { tile_id: 0 }; (width * height) as usize],
            width,
            height,
            tile_size,
            dirty: true,
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile_id: u32) {
        self.tiles[(y * self.width + x) as usize].tile_id = tile_id;
        self.dirty = true; // mark for re-upload
    }
}