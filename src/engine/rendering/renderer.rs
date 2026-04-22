use crate::rendering::camera::Camera;
use crate::rendering::vertex::Vertex;
use crate::rendering::instance::Instance;
use crate::rendering::texture::Texture;
use crate::rendering;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use winit::window::Window;

// ── Batch ─────────────────────────────────────────────────────────────────────

pub struct Batch {
    pub instances:       Vec<Instance>,
    instance_buffer:     wgpu::Buffer,
    instance_capacity:   usize,         // track buffer size to know when to reallocate
    num_instances:       u32,
    bind_group:          wgpu::BindGroup,
    _texture:            Texture,       // keeps GPU texture alive
}

// ── Renderer ──────────────────────────────────────────────────────────────────

pub struct Renderer {
    // ── wgpu core ────────────────────────────────────────────────────────────
    instance:          wgpu::Instance,
    adapter:           Option<wgpu::Adapter>,
    surface:           Option<wgpu::Surface<'static>>,
    pub device:        Option<wgpu::Device>,
    queue:             Option<wgpu::Queue>,
    config:            Option<wgpu::SurfaceConfiguration>,

    // ── pipeline ─────────────────────────────────────────────────────────────
    pipeline:          Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,

    // ── shared geometry ───────────────────────────────────────────────────────
    vertex_buffer:     Option<wgpu::Buffer>,
    index_buffer:      Option<wgpu::Buffer>,
    num_indices:       u32,

    // ── camera ───────────────────────────────────────────────────────────────
    pub camera:        Camera,
    camera_buffer:     Option<wgpu::Buffer>,

    // ── batches ───────────────────────────────────────────────────────────────
    pub batches:       Vec<Batch>,

    // ── tilemap ───────────────────────────────────────────────────────────────
    pub tilemaps: Vec<rendering::TilemapRenderer>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            instance:          wgpu::Instance::default(),
            adapter:           None,
            surface:           None,
            device:            None,
            queue:             None,
            config:            None,
            pipeline:          None,
            bind_group_layout: None,
            vertex_buffer:     None,
            index_buffer:      None,
            num_indices:       0,
            camera:            Camera::new(1080, 720),
            camera_buffer:     None,
            tilemaps:          Vec::new(),
            batches:           Vec::new()
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    pub async fn init(&mut self, window: Arc<Window>) {
        let size = window.inner_size();
        self.camera.zoom = 1.0;
        self.init_surface_and_device(window).await;
        self.init_geometry();
        self.init_pipeline(size);
        self.init_camera();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let config = self.config.as_mut().unwrap();
        config.width  = width.max(1);
        config.height = height.max(1);
        self.surface.as_ref().unwrap()
            .configure(self.device.as_ref().unwrap(), config);

        self.camera.viewport_width  = width;
        self.camera.viewport_height = height;
        self.update_camera();
    }

    pub fn update_camera(&self) {
        let matrix = self.camera.build_matrix();
        self.queue().write_buffer(
            self.camera_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&matrix),
        );
    }

    /// Creates a new tilemap renderer from raw PNG bytes and initial tile data.
    /// Returns the tilemap index for later access via `renderer.tilemaps[idx]`.
    /// Maximum of 255 tiles per texture
    pub fn create_tilemap(
        &mut self,
        tex_bytes: &[u8],
        tile_data: &[u8],
        width:     u32,
        height:    u32,
        tile_size: u32,
    ) -> usize {
        let mut tilemap = rendering::TilemapRenderer::new(
            self.device(),
            self.config.as_ref().unwrap().format,
            self.camera_buffer.as_ref().unwrap(),
        );

        tilemap.update(
            self.device.as_ref().unwrap(),
            self.queue.as_ref().unwrap(),
            self.camera_buffer.as_ref().unwrap(),
            tex_bytes,
            tile_data,
            width,
            height,
            tile_size,
        );

        self.tilemaps.push(tilemap);
        self.tilemaps.len() - 1
    }
    /// Creates a new batch from raw PNG bytes and an initial instance list.
    /// Returns the batch index for later access via `renderer.batches[idx]`.
    pub fn create_batch(&mut self, tex_bytes: &[u8], instances: Vec<Instance>) -> usize {
        let tex = Texture::from_bytes(
            self.device(), self.queue(), tex_bytes, "batch_texture",
        ).expect("Failed to load batch texture");

        let instance_buffer = self.make_instance_buffer(&instances);
        let bind_group      = self.make_bind_group(&tex);

        let batch = Batch {
            num_instances:     instances.len() as u32,
            instance_capacity: instances.len(),
            instances,
            instance_buffer,
            bind_group,
            _texture: tex,
        };

        self.batches.push(batch);
        self.batches.len() - 1
    }

    /// Replaces the texture on an existing batch (e.g. swapping a sprite sheet).
    pub fn set_batch_texture(&mut self, batch_idx: usize, tex_bytes: &[u8]) {
        let tex        = Texture::from_bytes(
            self.device(), self.queue(), tex_bytes, "batch_texture",
        ).expect("Failed to load batch texture");
        let bind_group = self.make_bind_group(&tex);

        let batch          = &mut self.batches[batch_idx];
        batch.bind_group   = bind_group;
        batch._texture     = tex;
    }

    pub fn render(&mut self) -> Result<(), String> {
        self.update_camera();
        self.upload_instances();

        let surface = self.surface.as_ref().unwrap();
        let device  = self.device();
        let queue   = self.queue();

        let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)    => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Lost          => return Err("Surface lost".into()),
            wgpu::CurrentSurfaceTexture::Timeout       => return Err("Surface timeout".into()),
            wgpu::CurrentSurfaceTexture::Outdated      => return Err("Surface outdated".into()),
            _                                          => return Err("Failed to acquire texture".into()),
        };

        let view    = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.record_render_pass(&mut enc, &view);

        queue.submit(std::iter::once(enc.finish()));
        output.present();
        Ok(())
    }
    // ── Initialization helpers ────────────────────────────────────────────────
    async fn init_surface_and_device(&mut self, window: Arc<Window>) {
        let surface = self.instance
            .create_surface(window)
            .expect("Failed to create surface");

        let adapter = self.instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Failed to create device");

        self.surface = Some(surface);
        self.adapter = Some(adapter);
        self.device  = Some(device);
        self.queue   = Some(queue);
    }

    fn init_geometry(&mut self) {
        let vertices = [
            Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 0.0] }, // top-left
            Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] }, // bottom-left
            Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 1.0] }, // bottom-right
            Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] }, // top-right
        ];
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
        self.num_indices = indices.len() as u32;

        self.vertex_buffer = Some(self.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage:    wgpu::BufferUsages::VERTEX,
        }));

        self.index_buffer = Some(self.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage:    wgpu::BufferUsages::INDEX,
        }));
    }

    fn init_pipeline(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let caps       = self.surface.as_ref().unwrap().get_capabilities(self.adapter.as_ref().unwrap());
        let format     = caps.formats[0];

        let alpha_mode = caps.alpha_modes[0];
        
        let config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width:        size.width.max(1),
            height:       size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,

            // Force overide for alpha fix later with proper caps impl
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        self.surface.as_ref().unwrap().configure(self.device(), &config);
        self.config = Some(config);

        let shader = self.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let bind_group_layout = self.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {             // 0 — texture
                    binding:    0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type:    wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {             // 1 — sampler
                    binding:    1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty:         wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count:      None,
                },
                wgpu::BindGroupLayoutEntry {             // 2 — camera uniform
                    binding:    2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = self.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:              Some("Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size:     0,
        });

        self.pipeline = Some(self.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("Quad Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module:              &shader,
                entry_point:         Some("vs_main"),
                buffers:             &[Vertex::layout(), Instance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module:              &shader,
                entry_point:         Some("fs_main"),

                // Hopefully allows for alpha blending
                targets:             &[Some(wgpu::ColorTargetState { 
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), 
                    write_mask: wgpu::ColorWrites::COLOR 
                })],
                compilation_options: Default::default(),
            }),
            primitive:      wgpu::PrimitiveState::default(),
            depth_stencil:  None,
            multisample:    wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache:          None,
        }));

        self.bind_group_layout = Some(bind_group_layout);
    }

    fn init_camera(&mut self) {
        let config = self.config.as_ref().unwrap();
        self.camera.viewport_width  = config.width;
        self.camera.viewport_height = config.height;

        let matrix = self.camera.build_matrix();
        self.camera_buffer = Some(self.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&matrix),
            usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }));
    }

    // ── Batch helpers ─────────────────────────────────────────────────────────

    fn make_instance_buffer(&self, instances: &[Instance]) -> wgpu::Buffer {
        // Allocate 2x requested capacity so moderate growth avoids reallocation
        let capacity = (instances.len() * 2).max(1);
        let buffer   = self.device().create_buffer(&wgpu::BufferDescriptor {
            label:              Some("Instance Buffer"),
            size:               (std::mem::size_of::<Instance>() * capacity) as u64,
            usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Write initial data
        self.queue().write_buffer(&buffer, 0, bytemuck::cast_slice(instances));
        buffer
    }

    fn make_bind_group(&self, tex: &Texture) -> wgpu::BindGroup {
        self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("Batch Bind Group"),
            layout:  self.bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding:  0,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::BindGroupEntry {
                    binding:  1,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                },
                wgpu::BindGroupEntry {
                    binding:  2,
                    resource: self.camera_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        })
    }

    // ── Per-frame helpers ─────────────────────────────────────────────────────
    pub fn update_tilemap(
        &mut self,
        idx:       usize,
        tex_bytes: &[u8],
        tile_data: &[u8],
        width:     u32,
        height:    u32,
        tile_size: u32,
    ) {
        self.tilemaps[idx].update(
            self.device.as_ref().unwrap(),
            self.queue.as_ref().unwrap(),
            self.camera_buffer.as_ref().unwrap(),
            tex_bytes,
            tile_data,
            width,
            height,
            tile_size,
        );
    }
    fn upload_instances(&mut self) {

        for batch in &mut self.batches {
            if batch.instances.is_empty() { continue; }
            // Reallocate buffer if instances grew beyond capacity
            if batch.instances.len() > batch.instance_capacity {
                let new_capacity    = batch.instances.len() * 2;
                batch.instance_buffer = self.device.as_ref().unwrap().create_buffer(
                    &wgpu::BufferDescriptor {
                        label:              Some("Instance Buffer (resized)"),
                        size:               (std::mem::size_of::<Instance>() * new_capacity) as u64,
                        usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }
                );
                batch.instance_capacity = new_capacity;
            }

            batch.num_instances = batch.instances.len() as u32;
            self.queue.as_ref().unwrap().write_buffer(
                &batch.instance_buffer,
                0,
                bytemuck::cast_slice(&batch.instances),
            );
        }
    }

    fn record_render_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load:  wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        // Draw all tilemaps before sprites
        for tilemap in &self.tilemaps {
            tilemap.record(&mut rpass);
        }
        

        // Sprites on top
        rpass.set_pipeline(self.pipeline.as_ref().unwrap());
        rpass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        rpass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);

        for batch in &self.batches {
            if batch.num_instances == 0 { continue; }
            rpass.set_bind_group(0, &batch.bind_group, &[]);
            rpass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
            rpass.draw_indexed(0..self.num_indices, 0, 0..batch.num_instances);
        }
    }

    // ── Convenience accessors ─────────────────────────────────────────────────

    fn device(&self) -> &wgpu::Device { self.device.as_ref().unwrap() }
    pub fn queue(&self)  -> &wgpu::Queue  { self.queue.as_ref().unwrap()  }
}