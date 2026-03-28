use bytemuck::{Pod, Zeroable};
 
// ── Vertex layout ──────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
 
impl Vertex {
    const ATTRS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2
    ];
 
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}
 
// A unit quad centred at the origin – two triangles, four vertices.
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 0.0] }, // top-left
    Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] }, // bottom-left
    Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 1.0] }, // bottom-right
    Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] }, // top-right
];
 
const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];