use crate::rendering::camera::Camera;
use crate::rendering::vertex::Vertex;
use crate::rendering::instance::Instance;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    _texture_view: Option<wgpu::TextureView>,
    _sampler: Option<wgpu::Sampler>,
    index_buffer: Option<wgpu::Buffer>,
    num_indices: u32,
    

    // instance buffer for per-instance data (like transforms)
    instance_buffer: Option<wgpu::Buffer>,// Gpu buffer
    pub instances: Vec<Instance>,// Cpu side data for modification
    num_instances: u32,


    // unused rn
    camera: Camera,
    camera_buffer: Option<wgpu::Buffer>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
            surface: None,
            device: None,
            queue: None,
            config: None,
            pipeline: None,
            vertex_buffer: None,

            //Instances for sprite
            instances: Vec::new(),
            instance_buffer: None, // gpu
            num_instances: 0,
            camera: Camera::new(1080, 720), // Replace with actual viewport dimensions
            camera_buffer: None,
            bind_group: None,
            _texture_view: None,
            _sampler: None,
            index_buffer: None,
            num_indices: 0,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(config) = &mut self.config {
            config.width = width.max(1);
            config.height = height.max(1);
            self.surface.as_ref().unwrap()
                .configure(self.device.as_ref().unwrap(), config);
        }
        self.camera.viewport_width = width;
        self.camera.viewport_height = height;
        self.update_camera(); // recalculate and reupload matrix
    }

    pub fn update_camera(&self) {
        let matrix = self.camera.build_matrix();
        self.queue.as_ref().unwrap().write_buffer(
            self.camera_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&matrix),
        );
    }
    pub async fn initialize(&mut self, window: Arc<Window>) {
        
        let size = window.inner_size();

        let surface = self.instance.create_surface(window).expect("Failed to create surface");
        let adapter = self.instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.unwrap();
        
        // !! Test instanceing a texture here !!
        let mut instances = vec![
            Instance { position: [-0.5, 0.0], size: [1.0, 1.0], uv_offset: [0.0, 0.0], uv_scale: [1.0, 1.0] },
            //Instance { position: [ 0.5, 0.0], size: [1.0, 1.0], uv_offset: [0.0, 0.0], uv_scale: [1.0, 1.0] },
        ];

        self.num_instances = instances.len() as u32;
        self.instances = instances;
        //instances.push(Instance{ position: [0.0, 0.0], size: [0.1, 0.1], uv_offset: [0.0, 0.0], uv_scale: [100.0, 1.0] });
        

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // COPY_DST lets you update it later
        });
        self.instance_buffer = Some(instance_buffer);
        // End of instance buffer test




        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Quad vertices with UV coords
        let vertices = [
            Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 0.0] }, // Top Left
            Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 1.0] }, // Bottom Left
            Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 1.0] }, // Bottom Right
            Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0] }, // Top Right
        ];

        // Two triangles forming a quad: (0,1,2) and (0,2,3)
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
        self.num_indices = indices.len() as u32;

        // Create buffers — only once each
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Checkerboard texture
        let tex_size = 64u32;
        let mut pixels = Vec::new();
        for y in 0..tex_size {
            for x in 0..tex_size {
                let is_white = ((x / 8) + (y / 8)) % 2 == 0;
                pixels.extend_from_slice(if is_white { &[255u8, 255, 255, 255] } else { &[255u8, 0, 255, 255] });
            }
        }

        let texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("Checkerboard Texture"),
                size: wgpu::Extent3d { width: tex_size, height: tex_size, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &pixels,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });


        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                // Texture
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
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Camera uniform
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
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"), // Debug label
            bind_group_layouts: &[Some(&bind_group_layout)], // Slice of BindGroupLayout references
            immediate_size: 0, // Used for immediate data (requires specific feature)
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout(), Instance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(config.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let camera_matrix = self.camera.build_matrix();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&camera_matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });


        
        //let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: camera_buffer.as_entire_binding() },
            ],
        });


        
        self.camera_buffer = Some(camera_buffer);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.pipeline = Some(pipeline);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.bind_group = Some(bind_group);
        self._texture_view = Some(texture_view);
        self._sampler = Some(sampler);

    }
    pub fn update_instances(&mut self) {
        let queue = self.queue.as_ref().unwrap();
        let buffer = self.instance_buffer.as_ref().unwrap();
        self.update_camera();

        // integrate with my ECS to update instance data based on entity transforms, etc.

        self.camera.position[0] += 0.01; // Move camera right over time
        self.camera.zoom -= 0.01; // Zoom in/out over time
        for i in 0..self.instances.len() {
            self.instances[i].position[0] += 0.00; // Move right over time
            self.instances[i].position[1] += 0.00;
            if self.instances[i].position[0] > 1.0 {
                self.instances[i].position[0] = -1.0;
                self.instances[i].position[1] = -1.0; // Wrap around
            }
        }


        queue.write_buffer(
            buffer,
            0, // Offset
            bytemuck::cast_slice(self.instances.as_slice()),
        );
    }
    pub fn render(&mut self) -> Result<(), String> {
        self.update_instances(); // Update instance data before rendering

        
        let surface = self.surface.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();

        let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Lost => return Err("Surface lost".into()),
            wgpu::CurrentSurfaceTexture::Timeout => return Err("Surface timeout".into()),
            wgpu::CurrentSurfaceTexture::Outdated => return Err("Surface outdated".into()),
            _ => return Err("Failed to acquire next surface texture".into()),
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            rpass.set_pipeline(self.pipeline.as_ref().unwrap());
            rpass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.as_ref().unwrap().slice(..)); // <-- slot 1
            rpass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.num_indices, 0, 0..self.num_instances); 
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}