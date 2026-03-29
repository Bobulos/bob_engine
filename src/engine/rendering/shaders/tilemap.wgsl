// ── Bindings ──────────────────────────────────────────────────────────────────
struct TilemapInfo {
    width:      u32,
    height:     u32,
    atlas_rows: u32,
    atlas_cols: u32,
    offset:     vec2<f32>,
    _pad:       vec2<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct TileUV {
    uv_offset: vec2<f32>,
    uv_scale:  vec2<f32>,
};

@group(0) @binding(0) var                    t_diffuse:  texture_2d<f32>;
@group(0) @binding(1) var                    s_diffuse:  sampler;
@group(0) @binding(2) var<uniform>           camera:     CameraUniform;
@group(0) @binding(3) var<storage, read>     tiles:      array<u32>;      // tile ID per cell
@group(0) @binding(4) var<storage, read>     tile_uvs:   array<TileUV>;   // UV per tile type
@group(0) @binding(5) var<uniform>           map_info:   TilemapInfo;

// ── Vertex output ──────────────────────────────────────────────────────────────

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       world_pos:     vec2<f32>,
};

// ── Vertex shader ──────────────────────────────────────────────────────────────
//
// No vertex buffer — geometry is generated from builtins.
//   vertex_index   0..5  → two triangles forming one quad
//   instance_index 0..W*H → which tile in the map
//
@vertex
fn vs_main(@builtin(vertex_index) vert_idx: u32) -> VertexOutput {
    // Single quad covering the entire map
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0,                    0.0),
        vec2<f32>(f32(map_info.width),    0.0),
        vec2<f32>(f32(map_info.width),    f32(map_info.height)),
        vec2<f32>(0.0,                    0.0),
        vec2<f32>(f32(map_info.width),    f32(map_info.height)),
        vec2<f32>(0.0,                    f32(map_info.height)),
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(positions[vert_idx] + map_info.offset, 0.0, 1.0);
    out.world_pos     = positions[vert_idx]; // pass to fragment
    return out;
}

// ── Fragment shader ────────────────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Which tile are we in?
    let tile_x = u32(floor(in.world_pos.x));
    let tile_y = u32(floor(in.world_pos.y));

    // Clamp to map bounds
    if tile_x >= map_info.width || tile_y >= map_info.height {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let tile_idx = tile_y * map_info.width + tile_x;
    let tile_id  = tiles[tile_idx];
    let uv       = tile_uvs[tile_id];

    // UV within the tile (0..1)
    let local_uv = fract(in.world_pos);
    let tex_coords = uv.uv_offset + vec2<f32>(local_uv.x, 1.0 - local_uv.y) * uv.uv_scale;

    return textureSample(t_diffuse, s_diffuse, tex_coords);
}