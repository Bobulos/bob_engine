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
    @location(0)       tex_coords:    vec2<f32>,
};

// ── Vertex shader ──────────────────────────────────────────────────────────────
//
// No vertex buffer — geometry is generated from builtins.
//   vertex_index   0..5  → two triangles forming one quad
//   instance_index 0..W*H → which tile in the map
//
@vertex
fn vs_main(
    @builtin(vertex_index)   vert_idx: u32,
    @builtin(instance_index) tile_idx: u32,
) -> VertexOutput {

    // Unit quad corners (two triangles, no index buffer needed)
    //   3──2
    //   │ /│
    //   │/ │
    //   0──1
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), // tri 0: bottom-left
        vec2<f32>(1.0, 0.0), // tri 0: bottom-right
        vec2<f32>(1.0, 1.0), // tri 0: top-right
        vec2<f32>(0.0, 0.0), // tri 1: bottom-left
        vec2<f32>(1.0, 1.0), // tri 1: top-right
        vec2<f32>(0.0, 1.0), // tri 1: top-left
    );

    // UV corners matching the positions above
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), // bottom-left  → UV bottom-left
        vec2<f32>(1.0, 1.0), // bottom-right → UV bottom-right
        vec2<f32>(1.0, 0.0), // top-right    → UV top-right
        vec2<f32>(0.0, 1.0), // bottom-left  → UV bottom-left
        vec2<f32>(1.0, 0.0), // top-right    → UV top-right
        vec2<f32>(0.0, 0.0), // top-left     → UV top-left
    );

    // Tile grid position from flat index
    let tile_x = tile_idx % map_info.width;
    let tile_y = tile_idx / map_info.width;

    // World position: each tile is 1 world unit; origin at (0,0)
    // Flip Y so tile_y=0 is at the top, matching typical tilemap convention
    let world_pos = positions[vert_idx] + vec2<f32>(
        f32(tile_x),
        f32(map_info.height - 1u - tile_y),
    )+map_info.offset;

    // Look up the UV region for this tile type
    let tile_id = tiles[tile_idx];
    let uv      = tile_uvs[tile_id];
    let tex_coords = uv.uv_offset + uvs[vert_idx] * uv.uv_scale;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.tex_coords    = tex_coords;
    return out;
}

// ── Fragment shader ────────────────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
