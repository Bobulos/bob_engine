#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub position: [f32; 2],  // world position
    pub size: [f32; 2],      // width, height scale
    pub uv_offset: [f32; 2], // for sprite sheets, else [0.0, 0.0]
    pub uv_scale: [f32; 2],  // for sprite sheets, else [1.0, 1.0]
}

impl Instance {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance, // <-- key difference from vertex buffers
            attributes: &[
                wgpu::VertexAttribute { shader_location: 2, format: wgpu::VertexFormat::Float32x2, offset: 0 },
                wgpu::VertexAttribute { shader_location: 3, format: wgpu::VertexFormat::Float32x2, offset: 8 },
                wgpu::VertexAttribute { shader_location: 4, format: wgpu::VertexFormat::Float32x2, offset: 16 },
                wgpu::VertexAttribute { shader_location: 5, format: wgpu::VertexFormat::Float32x2, offset: 24 },
            ],
        }
    }
}