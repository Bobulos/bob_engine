use crate::tilemap::{TileUV, TilemapInfo};
use crate::rendering::texture::Texture;
use wgpu::util::DeviceExt;

pub struct TilemapRenderer {
    pipeline:        wgpu::RenderPipeline,
    bind_group:      Option<wgpu::BindGroup>,
    layout:          wgpu::BindGroupLayout,
    tile_buffer:     wgpu::Buffer,
    tile_uv_buffer:  Option<wgpu::Buffer>,
    info_buffer:     Option<wgpu::Buffer>,
    tileset_texture: Option<Texture>,   // keeps GPU texture alive
    pub width:       u32,
    pub height:      u32,
    pub position: [f32; 2],
}

impl TilemapRenderer {
    pub fn new(
        device:        &wgpu::Device,
        format:        wgpu::TextureFormat,
        camera_buffer: &wgpu::Buffer,
    ) -> Self {
        const MAX_TILES: u64 = 256 * 256;

        let tile_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("Tile Storage Buffer"),
            size:               MAX_TILES * 4,
            usage:              wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("Tilemap BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding:    0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type:    wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty:         wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count:      None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    5,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("Tilemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/tilemap.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:              Some("Tilemap Pipeline Layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size:     0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("Tilemap Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module:              &shader,
                entry_point:         Some("vs_main"),
                buffers:             &[],   // no vertex buffer — geometry is in the shader
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module:              &shader,
                entry_point:         Some("fs_main"),
                targets:             &[Some(format.into())],
                compilation_options: Default::default(),
            }),
            primitive:      wgpu::PrimitiveState::default(),
            depth_stencil:  None,
            multisample:    wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache:          None,
        });

        Self {
            pipeline,
            layout,
            position:        [0.0, 0.0],
            tile_buffer,
            bind_group:      None,
            tile_uv_buffer:  None,
            info_buffer:     None,
            tileset_texture: None,
            width:           0,
            height:          0,
        }
    }
    pub fn flush_position(&self, queue: &wgpu::Queue) {
        if let Some(buf) = &self.info_buffer {
            // Write just the offset portion — offset starts at byte 16
            queue.write_buffer(buf, 16, bytemuck::cast_slice(&self.position));
        }
    }
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.position = [x, y];
    }

    pub fn move_by(&mut self, dx: f32, dy: f32) {
        self.position[0] += dx;
        self.position[1] += dy;
    }

    /// Call whenever tile data changes — skips GPU upload if nothing is dirty.
    pub fn update(
        &mut self,
        device:        &wgpu::Device,
        queue:         &wgpu::Queue,
        camera_buffer: &wgpu::Buffer,
        tex_bytes:     &[u8],
        tile_data:     &[u32],
        width:         u32,
        height:        u32,
        tile_size:     u32,
    ) {
        // Load tileset texture
        let tex = Texture::from_bytes(device, queue, tex_bytes, "tilemap_texture")
            .expect("Failed to load tilemap texture");

        let atlas_cols = tex.texture.width()  / tile_size;
        let atlas_rows = tex.texture.height() / tile_size;
        let atlas_uvs  = TileUV::generate_atlas(atlas_cols, atlas_rows);

        // UV buffer — always recreate (atlas layout may have changed)
        self.tile_uv_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("Tile UV Buffer"),
            contents: bytemuck::cast_slice(&atlas_uvs),
            usage:    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        }));

        // Info buffer — write_buffer if it exists, create if not
        let info = TilemapInfo { width, height, atlas_rows, atlas_cols, offset: [0.0,0.0], _pad: [0.0; 2], };
        match &self.info_buffer {
            Some(buf) => queue.write_buffer(buf, 0, bytemuck::cast_slice(&[info])),
            None => {
                self.info_buffer = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label:    Some("Tilemap Info Buffer"),
                        contents: bytemuck::cast_slice(&[info]),
                        usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    },
                ));
            }
        }
        // Tile index data
        queue.write_buffer(&self.tile_buffer, 0, bytemuck::cast_slice(tile_data));

        // Rebuild bind group (texture or UV layout may have changed)
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("Tilemap Bind Group"),
            layout:  &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&tex.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: camera_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: self.tile_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: self.tile_uv_buffer.as_ref().unwrap().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: self.info_buffer.as_ref().unwrap().as_entire_binding() },
            ],
        }));

        self.tileset_texture = Some(tex);
        self.width  = width;
        self.height = height;
    }

    /// Called from the renderer
    pub fn record(
        &self,
        rpass: &mut wgpu::RenderPass,
    ) {
        let (Some(bg), w, h) = (&self.bind_group, self.width, self.height) else { return };
        if w == 0 || h == 0 { return; }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, bg, &[]);
        // 6 vertices hardcoded in shader (two triangles), one instance per tile
        rpass.draw(0..6, 0..1);
    }
}