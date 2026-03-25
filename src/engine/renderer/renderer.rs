use sdl3::video::Window;
use crate::rendering::camera::Camera;
use crate::rendering::vertex::Vertex;
use wgpu::util::DeviceExt;

const QUAD_VERTICES: [Vertex; 6] = [
    Vertex { position: [-0.5, -0.5, 0.0], uv: [0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, 0.0], uv: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5, 0.0], uv: [1.0, 0.0] },

    Vertex { position: [-0.5, -0.5, 0.0], uv: [0.0, 1.0] },
    Vertex { position: [ 0.5,  0.5, 0.0], uv: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5, 0.0], uv: [0.0, 0.0] },
];

pub struct Renderer {
    window: Window,

    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,

    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,

    camera: Camera,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            instance: wgpu::Instance::default(),
            surface: None,
            device: None,
            queue: None,
            config: None,
            pipeline: None,
            vertex_buffer: None,
            camera: Camera::new(1.0, 1080, 720, 32),
        }
    }

    pub async fn initialize(&mut self) {
        // Requires  feature "raw-window-handle" in Cargo.toml:
        //   sdl3 = { version = "0.17", features = ["raw-window-handle"] }
        // With that feature, sdl3::video::Window implements HasWindowHandle + HasDisplayHandle.
        let surface = unsafe {
            self.instance.create_surface_unsafe(
                wgpu::SurfaceTargetUnsafe::from_window(&self.window).unwrap()
            )
        }.unwrap();

        let adapter = self.instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // wgpu 29: request_device takes only 1 argument
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let (width, height) = self.window.size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Build vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad VBO"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Build pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        // wgpu 29: push_constant_ranges replaced by immediate_size
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),   // wgpu 29: Option<&str>
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),   // wgpu 29: Option<&str>
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            // wgpu 29: MultisampleState has no multiview_mask field
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // wgpu 29: multiview_mask replaces multiview: None
            multiview_mask: None,
            cache: None,
        });

        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.pipeline = Some(pipeline);
        self.vertex_buffer = Some(vertex_buffer);
    }

    pub fn render(&mut self) {
        let surface = self.surface.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let pipeline = self.pipeline.as_ref().unwrap();
        let vertex_buffer = self.vertex_buffer.as_ref().unwrap();

        // wgpu 29: get_current_texture() returns CurrentSurfaceTexture enum
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            _ => return,
        };

        let view = frame.texture.create_view(&Default::default());

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None }
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                multiview_mask: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(pipeline);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}