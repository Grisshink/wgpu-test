use ab_glyph::{Font, FontRef};
use color::{ColorSpace, Hsl};
use imageproc::{drawing::{draw_text_mut, text_size}, image::{Rgba, RgbaImage}};
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler, 
    event::WindowEvent, 
    event_loop::{
        ActiveEventLoop,
        EventLoop,
    }, 
    window::{Window, WindowId},
};

use std::sync::Arc;

fn get_text(text: &str) -> RgbaImage {
    let font = FontRef::try_from_slice(include_bytes!("./IosevkaTermNerdFont-Bold.ttf")).unwrap();
    let scale = font.pt_to_px_scale(192.0).unwrap();
    let (w, _h) = text_size(scale, &font, text);

    let mut img = RgbaImage::new(w, scale.y as u32);
    draw_text_mut(&mut img, Rgba([255, 255, 255, 255]), 0, 0, scale, &font, text);

    img
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    aspect: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Colors {
    bg_color: [f32; 3],
    _pad: u32,
    fg_color: [f32; 3],
    _pad2: u32,
}

impl Colors {
    fn new(bg_color: [f32; 3], fg_color: [f32; 3]) -> Self {
        Colors { bg_color, _pad: 0, fg_color, _pad2: 0 }
    }
}

const QUAD: &[Vertex] = &[
    Vertex { pos: [-1.0, -1.0], uv: [0.0, 0.0] },
    Vertex { pos: [-1.0,  1.0], uv: [0.0, 1.0] },
    Vertex { pos: [ 1.0, -1.0], uv: [1.0, 0.0] },
    Vertex { pos: [ 1.0,  1.0], uv: [1.0, 1.0] },
];

struct PipelineBuilder<'a> {
    device: &'a wgpu::Device, 
    bind_groups: Vec<&'a wgpu::BindGroupLayout>, 
    blending: wgpu::BlendState,
    buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    shader_code: &'a str,
    color_format: wgpu::TextureFormat,
}

impl<'a> PipelineBuilder<'a> {
    fn new(device: &'a wgpu::Device, color_format: wgpu::TextureFormat, shader_code: &'a str) -> Self {
        PipelineBuilder { 
            color_format,
            device,
            bind_groups: vec![],
            buffers: vec![],
            shader_code,
            blending: wgpu::BlendState::REPLACE,
        }
    }

    fn with_blending(mut self, blending: wgpu::BlendState) -> Self {
        self.blending = blending;
        self
    }

    fn with_bind_group(mut self, bind_group: &'a wgpu::BindGroupLayout) -> Self {
        self.bind_groups.push(bind_group);
        self
    }

    fn with_buffer(mut self, buffer: wgpu::VertexBufferLayout<'a>) -> Self {
        self.buffers.push(buffer);
        self
    }

    fn build(self) -> wgpu::RenderPipeline {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.shader_code.into()),
        });
        
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("some render pipeline layout"),
            bind_group_layouts: self.bind_groups.as_slice(),
            push_constant_ranges: &[],
        });

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("Some render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: self.buffers.as_slice(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState { 
                        format: self.color_format,
                        blend: Some(self.blending),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { 
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }
}

fn get_back_texture(device: &wgpu::Device, size: (u32, u32)) -> (wgpu::Texture, wgpu::BindGroup) {
    let texture_size = wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::wgt::TextureDescriptor { 
        label: Some("back_texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let text_texture_sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor { 
        label: Some("text_texture_sampler"),
        address_mode_u: wgpu::AddressMode::MirrorRepeat,
        address_mode_v: wgpu::AddressMode::MirrorRepeat,
        address_mode_w: wgpu::AddressMode::MirrorRepeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
        label: Some("uniform_bind_group_layout"), 
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
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
        label: Some("back_texture_bind_group"),
        layout: &group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&text_texture_sampler),
            },
        ],
    });

    (texture, bind_group)
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,

    pipeline: wgpu::RenderPipeline,
    pipeline_post: wgpu::RenderPipeline,
    pipeline_text: wgpu::RenderPipeline,

    buffer: wgpu::Buffer,
    text_vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    _color_buffer: wgpu::Buffer,

    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    back_texture_bind_group: wgpu::BindGroup,

    back_texture: wgpu::Texture,
    text_texture: wgpu::Texture,

    timer: std::time::Instant,

    window: Arc<Window>,
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let window_size = window.inner_size();

        // Prepare GPU

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { 
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::PRIMARY)
            .into_iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .unwrap();

        let adapter_info = adapter.get_info();
        log::info!("Using adapter: {}, Backend: {:?}", adapter_info.name, adapter_info.backend);

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Load vertex buffer

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Load uniform buffers

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                time: 0.0,
                aspect: config.width as f32 / config.height as f32,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut rng = rand::rng();

        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Color buffer"),
            contents: bytemuck::cast_slice(&[Colors::new(
                Hsl::to_linear_srgb([rng.random_range(0.0..360.0), rng.random_range(0.0..100.0), 10.0]),
                Hsl::to_linear_srgb([rng.random_range(0.0..360.0), rng.random_range(0.0..100.0), 70.0]),
            )]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let text_img = get_text("Абоба");
        let text_dimensions = text_img.dimensions();
        let text_size = wgpu::Extent3d {
            width: text_dimensions.0,
            height: text_dimensions.1,
            depth_or_array_layers: 1,
        };

        let text_ndc_size = (text_size.width  as f32 / window_size.width  as f32, 
                             text_size.height as f32 / window_size.height as f32);

        let text_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text vertex buffer"),
            contents: bytemuck::cast_slice(&[
                Vertex { pos: [-1.0 * text_ndc_size.0, -1.0 * text_ndc_size.1], uv: [0.0, 0.0] },
                Vertex { pos: [-1.0 * text_ndc_size.0,  1.0 * text_ndc_size.1], uv: [0.0, 1.0] },
                Vertex { pos: [ 1.0 * text_ndc_size.0, -1.0 * text_ndc_size.1], uv: [1.0, 0.0] },
                Vertex { pos: [ 1.0 * text_ndc_size.0,  1.0 * text_ndc_size.1], uv: [1.0, 1.0] },
            ]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        
        let text_texture = device.create_texture(&wgpu::wgt::TextureDescriptor { 
            label: Some("text_texture"),
            size: text_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &text_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &text_img,
            wgpu::TexelCopyBufferLayout { 
                offset: 0,
                bytes_per_row: Some(4 * text_dimensions.0),
                rows_per_image: Some(text_dimensions.1),
            },
            text_size,
        );

        let text_texture_view = text_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let text_texture_sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor { 
            label: Some("text_texture_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("uniform_bind_group_layout"), 
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
            ],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&text_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&text_texture_sampler),
                },
            ],
        });

        let (back_texture, back_texture_bind_group) = get_back_texture(&device, (window_size.width, window_size.height));

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("uniform_bind_group_layout"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None, 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None, 
                    },
                    count: None,
                },
            ],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: color_buffer.as_entire_binding(),
                },
            ],
        });

        // Load shader and define pipeline

        let pipeline = PipelineBuilder::new(&device, back_texture.format(), include_str!("./shader.wgsl"))
            .with_buffer(Vertex::desc())
            .with_bind_group(&uniform_bind_group_layout)
            .build();

        let text_blending = wgpu::BlendComponent { 
            src_factor: wgpu::BlendFactor::OneMinusDst,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            ..Default::default()
        };

        let pipeline_text = PipelineBuilder::new(&device, back_texture.format(), include_str!("./shader_text.wgsl"))
            .with_buffer(Vertex::desc())
            .with_bind_group(&texture_bind_group_layout)
            .with_blending(wgpu::BlendState { 
                color: text_blending,
                alpha: text_blending, 
            })
            .build();

        let pipeline_post = PipelineBuilder::new(&device, config.format, include_str!("./shader_post.wgsl"))
            .with_buffer(Vertex::desc())
            .with_bind_group(&uniform_bind_group_layout)
            .with_bind_group(&texture_bind_group_layout)
            .build();

        Ok(State {
            window, surface, device,
            queue, config, pipeline, buffer,
            uniform_buffer, uniform_bind_group,
            texture_bind_group, pipeline_post, pipeline_text,
            text_vertex_buffer, text_texture,
            back_texture, back_texture_bind_group,
            is_surface_configured: false,
            timer: std::time::Instant::now(),
            _color_buffer: color_buffer,
        })
    }

    fn on_resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.back_texture.destroy();

            let (tex, tex_group) = get_back_texture(&self.device, (w, h));
            self.back_texture = tex;
            self.back_texture_bind_group = tex_group;

            let (text_width, text_height) = (self.text_texture.width(), self.text_texture.height());

            let text_ndc_size = (text_width  as f32 / w as f32, 
                                 text_height as f32 / h as f32);

            self.queue.write_buffer(&self.text_vertex_buffer, 0, bytemuck::cast_slice(&[
                Vertex { pos: [-1.0 * text_ndc_size.0, -1.0 * text_ndc_size.1], uv: [0.0, 0.0] },
                Vertex { pos: [-1.0 * text_ndc_size.0,  1.0 * text_ndc_size.1], uv: [0.0, 1.0] },
                Vertex { pos: [ 1.0 * text_ndc_size.0, -1.0 * text_ndc_size.1], uv: [1.0, 0.0] },
                Vertex { pos: [ 1.0 * text_ndc_size.0,  1.0 * text_ndc_size.1], uv: [1.0, 1.0] },
            ]));
        }
    }

    fn on_draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let timer = std::time::Instant::now();

        if !self.is_surface_configured {
            return Ok(());
        }

        // Update uniform buffer
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[Uniforms {
            time: self.timer.elapsed().as_secs_f32(),
            aspect: self.config.width as f32 / self.config.height as f32,
        }]));

        let output = self.surface.get_current_texture()?;
        let back_view = self.back_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let view = output.texture.create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Some encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Some render pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment { 
                        view: &back_view,
                        resolve_target: None,
                        ops: wgpu::Operations { 
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), 
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..QUAD.len() as u32, 0..1);

            render_pass.set_pipeline(&self.pipeline_text);
            render_pass.set_vertex_buffer(0, self.text_vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        {
            let mut render_pass_post = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Post render pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment { 
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { 
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), 
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass_post.set_pipeline(&self.pipeline_post);
            render_pass_post.set_vertex_buffer(0, self.buffer.slice(..));
            render_pass_post.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass_post.set_bind_group(1, &self.back_texture_bind_group, &[]);
            render_pass_post.draw(0..QUAD.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        let frame_time = timer.elapsed();
        let max_time = std::time::Duration::from_secs_f64(1.0 / 60.0);

        if frame_time < max_time {
            std::thread::sleep(max_time - frame_time);
        }

        self.window.request_redraw();

        Ok(())
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Sus window");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.on_resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                match state.on_draw() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let window_size = state.window.inner_size();
                        state.on_resize(window_size.width, window_size.height);
                    },
                    Err(e) => {
                        log::error!("Unable to render shit: {e}");
                    },
                }
            },
            _ => {},
        }
    }

}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut app = App::default();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;

    Ok(())
}
