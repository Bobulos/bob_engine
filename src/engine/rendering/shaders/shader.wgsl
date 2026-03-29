@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var<uniform> camera: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(2) i_pos: vec2<f32>,
    @location(3) i_size: vec2<f32>,
    @location(4) i_uv_offset: vec2<f32>,
    @location(5) i_uv_scale: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) v_pos: vec2<f32>,
    @location(1) v_uv: vec2<f32>,
    @location(2) i_pos: vec2<f32>,
    @location(3) i_size: vec2<f32>,
    @location(4) i_uv_offset: vec2<f32>,
    @location(5) i_uv_scale: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Scale quad by instance size and move to instance position
    let world_pos = (v_pos * i_size) + i_pos;
    out.clip_position = camera * vec4<f32>(world_pos, 0.0, 1.0);

    // Calculate which part of the texture to show
    // New UV = (Original Quad UV * Scale) + Offset
    out.tex_coords = (v_uv * i_uv_scale) + i_uv_offset;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture using the interpolated UVs
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}