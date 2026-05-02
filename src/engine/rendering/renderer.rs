// rendering/renderer.rs

use crate::rendering;
use crate::rendering::camera::Camera;
use crate::rendering::instance::Instance;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

// ── PipelineKey ───────────────────────────────────────────────────────────────

/// Identifies a compiled render pipeline. Add variants here for each new shader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PipelineKey {
    /// Standard alpha-blended sprite shader (the original).
    Default,
    /// Additive blending — good for particles, glows, fire.
    Additive,
    /// Custom/user-registered pipeline identified by an arbitrary string.
    Custom(String),
}

// ── Batch ─────────────────────────────────────────────────────────────────────

pub struct Batch {
    pub instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
    num_instances: u32,
    bind_group: wgpu::BindGroup,
    _texture: Texture,
    /// Which pipeline this batch renders with.
    pub pipeline_key: PipelineKey,
}

// ── Renderer ──────────────────────────────────────────────────────────────────

pub struct Renderer {
    // ── wgpu core ────────────────────────────────────────────────────────────
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    surface: Option<wgpu::Surface<'static>>,
    pub device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,

    // ── pipelines ─────────────────────────────────────────────────────────────
    /// All compiled pipelines, keyed by PipelineKey.
    pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    pipeline_layout: Option<wgpu::PipelineLayout>,

    // ── shared geometry ───────────────────────────────────────────────────────
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    num_indices: u32,

    // ── camera ───────────────────────────────────────────────────────────────
    pub camera: Camera,
    camera_buffer: Option<wgpu::Buffer>,

    // ── batches ───────────────────────────────────────────────────────────────
    pub batches: Vec<Batch>,

    // ── tilemap ───────────────────────────────────────────────────────────────
    pub tilemaps: Vec<rendering::TilemapRenderer>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
            adapter: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            pipelines: HashMap::new(),
            bind_group_layout: None,
            pipeline_layout: None,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            camera: Camera::new(1080, 720),
            camera_buffer: None,
            tilemaps: Vec::new(),
            batches: Vec::new(),
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
        config.width = width.max(1);
        config.height = height.max(1);
        self.surface
            .as_ref()
            .unwrap()
            .configure(self.device.as_ref().unwrap(), config);

        self.camera.viewport_width = width;
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

    /// Registers a new pipeline from raw WGSL source with a given blend state.
    ///
    /// # Example — register a greyscale shader at startup:
    /// ```rust
    /// renderer.register_pipeline(
    ///     PipelineKey::Custom("greyscale".into()),
    ///     include_str!("shaders/greyscale.wgsl"),
    ///     wgpu::BlendState::ALPHA_BLENDING,
    /// );
    /// ```
    pub fn register_pipeline(&mut self, key: PipelineKey, wgsl: &str, blend: wgpu::BlendState) {
        let pipeline = self.build_pipeline(wgsl, blend);
        self.pipelines.insert(key, pipeline);
    }

    /// Creates a new batch. `pipeline_key` selects which shader it renders with;
    /// pass `None` to use the default alpha-blended sprite pipeline.
    pub fn create_batch(
        &mut self,
        tex_bytes: &[u8],
        instances: Vec<Instance>,
        pipeline_key: Option<PipelineKey>,
    ) -> usize {
        let tex = Texture::from_bytes(self.device(), self.queue(), tex_bytes, "batch_texture")
            .expect("Failed to load batch texture");

        let instance_buffer = self.make_instance_buffer(&instances);
        let bind_group = self.make_bind_group(&tex);

        let batch = Batch {
            num_instances: instances.len() as u32,
            instance_capacity: instances.len(),
            instances,
            instance_buffer,
            bind_group,
            _texture: tex,
            pipeline_key: pipeline_key.unwrap_or(PipelineKey::Default),
        };

        self.batches.push(batch);
        self.batches.len() - 1
    }

    /// Replaces a batch at `idx`, optionally switching its pipeline.
    /// Pass `None` for `pipeline_key` to keep the existing batch's pipeline.
    pub fn replace_batch(
        &mut self,
        idx: usize,
        tex_bytes: &[u8],
        instances: Vec<Instance>,
        pipeline_key: Option<PipelineKey>,
    ) {
        let tex = Texture::from_bytes(self.device(), self.queue(), tex_bytes, "batch_texture")
            .expect("Failed to load batch texture");

        let instance_buffer = self.make_instance_buffer(&instances);
        let bind_group = self.make_bind_group(&tex);
        let key = pipeline_key.unwrap_or_else(|| self.batches[idx].pipeline_key.clone());

        self.batches[idx] = Batch {
            num_instances: instances.len() as u32,
            instance_capacity: instances.len(),
            instances,
            instance_buffer,
            bind_group,
            _texture: tex,
            pipeline_key: key,
        };
    }

    /// Swaps the pipeline on an existing batch without touching its texture or instances.
    pub fn set_batch_pipeline(&mut self, batch_idx: usize, key: PipelineKey) {
        self.batches[batch_idx].pipeline_key = key;
    }

    pub fn set_batch_texture(&mut self, batch_idx: usize, tex_bytes: &[u8]) {
        let tex = Texture::from_bytes(self.device(), self.queue(), tex_bytes, "batch_texture")
            .expect("Failed to load batch texture");
        let bind_group = self.make_bind_group(&tex);

        let batch = &mut self.batches[batch_idx];
        batch.bind_group = bind_group;
        batch._texture = tex;
    }

    pub fn render(&mut self) -> Result<(), String> {
        self.update_camera();
        self.upload_instances();

        let surface = self.surface.as_ref().unwrap();
        let device = self.device();
        let queue = self.queue();

        let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Lost => return Err("Surface lost".into()),
            wgpu::CurrentSurfaceTexture::Timeout => return Err("Surface timeout".into()),
            wgpu::CurrentSurfaceTexture::Outdated => return Err("Surface outdated".into()),
            _ => return Err("Failed to acquire texture".into()),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.record_render_pass(&mut enc, &view);

        queue.submit(std::iter::once(enc.finish()));
        output.present();
        Ok(())
    }

    // ── Initialization ────────────────────────────────────────────────────────

    async fn init_surface_and_device(&mut self, window: Arc<Window>) {
        let surface = self
            .instance
            .create_surface(window)
            .expect("Failed to create surface");

        let adapter = self
            .instance
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
        self.device = Some(device);
        self.queue = Some(queue);
    }

    fn init_geometry(&mut self) {
        let vertices = [
            Vertex {
                position: [-0.5, 0.5],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5],
                tex_coords: [1.0, 0.0],
            },
        ];
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
        self.num_indices = indices.len() as u32;

        self.vertex_buffer = Some(self.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
        self.index_buffer = Some(self.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        ));
    }

    fn init_pipeline(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let caps = self
            .surface
            .as_ref()
            .unwrap()
            .get_capabilities(self.adapter.as_ref().unwrap());
        let format = caps.formats[0];
        let alpha_mode = caps.alpha_modes[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        self.surface
            .as_ref()
            .unwrap()
            .configure(self.device(), &config);
        self.config = Some(config);

        // ── Shared bind group layout ──────────────────────────────────────────
        let bind_group_layout =
            self.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            self.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });

        self.bind_group_layout = Some(bind_group_layout);
        self.pipeline_layout = Some(pipeline_layout);

        // ── Built-in pipelines ────────────────────────────────────────────────
        let default_pipeline = self.build_pipeline(
            include_str!("shaders/shader.wgsl"),
            wgpu::BlendState::ALPHA_BLENDING,
        );
        self.pipelines
            .insert(PipelineKey::Default, default_pipeline);

        // Additive blend: src*1 + dst*1  (great for particles/glows)
        let additive_pipeline = self.build_pipeline(
            include_str!("shaders/shader.wgsl"), // same WGSL, different blend
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
        );
        self.pipelines
            .insert(PipelineKey::Additive, additive_pipeline);
    }

    /// Compiles a render pipeline from WGSL source + a blend state.
    /// Reuses the shared bind-group layout and pipeline layout.
    fn build_pipeline(&self, wgsl: &str, blend: wgpu::BlendState) -> wgpu::RenderPipeline {
        let format = self.config.as_ref().unwrap().format;

        let shader = self
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl.into()),
            });

        self.device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(self.pipeline_layout.as_ref().unwrap()),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::layout(), Instance::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::COLOR,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
    }

    fn init_camera(&mut self) {
        let config = self.config.as_ref().unwrap();
        self.camera.viewport_width = config.width;
        self.camera.viewport_height = config.height;

        let matrix = self.camera.build_matrix();
        self.camera_buffer = Some(self.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&matrix),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));
    }

    // ── Tilemap ───────────────────────────────────────────────────────────────

    pub fn create_tilemap(
        &mut self,
        tex_bytes: &[u8],
        tile_data: &[u8],
        width: u32,
        height: u32,
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

    pub fn update_tilemap(
        &mut self,
        idx: usize,
        tex_bytes: &[u8],
        tile_data: &[u8],
        width: u32,
        height: u32,
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

    // ── Per-frame ─────────────────────────────────────────────────────────────

    fn upload_instances(&mut self) {
        for batch in &mut self.batches {
            if batch.instances.is_empty() {
                continue;
            }

            if batch.instances.len() > batch.instance_capacity {
                let new_capacity = batch.instances.len() * 2;
                batch.instance_buffer =
                    self.device
                        .as_ref()
                        .unwrap()
                        .create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Instance Buffer (resized)"),
                            size: (std::mem::size_of::<Instance>() * new_capacity) as u64,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
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
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        // Tilemaps first (background)
        for tilemap in &self.tilemaps {
            tilemap.record(&mut rpass);
        }

        // Sprites — grouped by pipeline to minimise state changes
        rpass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        rpass.set_index_buffer(
            self.index_buffer.as_ref().unwrap().slice(..),
            wgpu::IndexFormat::Uint16,
        );

        let mut active_key: Option<&PipelineKey> = None;

        for batch in &self.batches {
            if batch.num_instances == 0 {
                continue;
            }

            // Only switch pipeline when the key actually changes
            if active_key != Some(&batch.pipeline_key) {
                let pipeline = self
                    .pipelines
                    .get(&batch.pipeline_key)
                    .expect("Batch references an unregistered PipelineKey");
                rpass.set_pipeline(pipeline);
                active_key = Some(&batch.pipeline_key);
            }

            rpass.set_bind_group(0, &batch.bind_group, &[]);
            rpass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
            rpass.draw_indexed(0..self.num_indices, 0, 0..batch.num_instances);
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_instance_buffer(&self, instances: &[Instance]) -> wgpu::Buffer {
        let capacity = (instances.len() * 2).max(1);
        let buffer = self.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<Instance>() * capacity) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue()
            .write_buffer(&buffer, 0, bytemuck::cast_slice(instances));
        buffer
    }

    fn make_bind_group(&self, tex: &Texture) -> wgpu::BindGroup {
        self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Batch Bind Group"),
            layout: self.bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.camera_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        })
    }

    fn device(&self) -> &wgpu::Device {
        self.device.as_ref().unwrap()
    }
    pub fn queue(&self) -> &wgpu::Queue {
        self.queue.as_ref().unwrap()
    }
}
