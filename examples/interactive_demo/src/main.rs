//! Interactive Globe Demo — Procedural Earth with atmosphere, city lights, and entity markers.
//!
//! Left drag = orbit, Scroll = zoom, Escape = quit

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    light_dir: [f32; 4],
    params: [f32; 4], // x=time, y=metallic, z=roughness, w=is_atmosphere
}

fn generate_sphere(
    segments: u32,
    rings: u32,
    radius: f32,
    color: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let pi = std::f32::consts::PI;

    for ring in 0..=rings {
        let theta = pi * ring as f32 / rings as f32;
        for seg in 0..=segments {
            let phi = 2.0 * pi * seg as f32 / segments as f32;
            let x = phi.cos() * theta.sin();
            let y = theta.cos();
            let z = phi.sin() * theta.sin();
            vertices.push(Vertex {
                position: [x * radius, y * radius, z * radius],
                normal: [x, y, z],
                color,
            });
        }
    }
    for ring in 0..rings {
        for seg in 0..segments {
            let cur = ring * (segments + 1) + seg;
            let next = cur + segments + 1;
            indices.extend_from_slice(&[cur, next, cur + 1, cur + 1, next, next + 1]);
        }
    }
    (vertices, indices)
}

fn generate_entity_markers(count: usize, globe_radius: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let pi = std::f32::consts::PI;

    for i in 0..count {
        // Scatter on globe surface using fibonacci sphere
        let golden = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let theta = (2.0 * pi * i as f32) / golden;
        let phi = (1.0 - 2.0 * (i as f32 + 0.5) / count as f32).acos();

        let x = phi.sin() * theta.cos();
        let y = phi.cos();
        let z = phi.sin() * theta.sin();

        let pos = glam::Vec3::new(x, y, z);
        let up = pos;
        let right = pos.cross(glam::Vec3::Y).normalize_or_zero();
        let fwd = right.cross(up).normalize_or_zero();

        let r = globe_radius + 0.005;
        let size = 0.008;

        let base = vertices.len() as u32;

        // Color by "threat level"
        let color = match i % 4 {
            0 => [1.0_f32, 0.15, 0.1], // hostile red
            1 => [0.1, 0.5, 1.0],      // friendly blue
            2 => [1.0, 0.85, 0.1],     // unknown yellow
            _ => [0.2, 0.9, 0.3],      // neutral green
        };

        // Small diamond shape on surface
        let center = pos * r;
        let top = center + glam::Vec3::from(up) * size * 1.5;
        let bot = center - glam::Vec3::from(up) * size * 0.5;
        let left = center - glam::Vec3::from(right) * size;
        let rght = center + glam::Vec3::from(right) * size;
        let front = center + glam::Vec3::from(fwd) * size;
        let back = center - glam::Vec3::from(fwd) * size;

        let n = up;
        for p in [top, bot, left, rght, front, back] {
            vertices.push(Vertex {
                position: p.to_array(),
                normal: n.to_array(),
                color,
            });
        }

        // 8 triangles forming a double pyramid
        indices.extend_from_slice(&[
            base,
            base + 2,
            base + 4, // top-left-front
            base,
            base + 4,
            base + 3, // top-front-right
            base,
            base + 3,
            base + 5, // top-right-back
            base,
            base + 5,
            base + 2, // top-back-left
            base + 1,
            base + 4,
            base + 2, // bot-front-left
            base + 1,
            base + 3,
            base + 4, // bot-right-front
            base + 1,
            base + 5,
            base + 3, // bot-back-right
            base + 1,
            base + 2,
            base + 5, // bot-left-back
        ]);
    }
    (vertices, indices)
}

struct OrbitCamera {
    distance: f32,
    phi: f32,
    theta: f32,
}

impl OrbitCamera {
    fn new() -> Self {
        Self {
            distance: 3.0,
            phi: 1.2,
            theta: 0.0,
        }
    }

    fn position(&self) -> glam::Vec3 {
        let x = self.distance * self.phi.sin() * self.theta.sin();
        let y = self.distance * self.phi.cos();
        let z = self.distance * self.phi.sin() * self.theta.cos();
        glam::Vec3::new(x, y, z)
    }

    fn orbit(&mut self, dx: f32, dy: f32) {
        self.theta += dx * 0.005;
        self.phi = (self.phi - dy * 0.005).clamp(0.2, std::f32::consts::PI - 0.2);
    }

    fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (1.0 - delta * 0.08)).clamp(1.3, 8.0);
    }
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    globe_pipeline: wgpu::RenderPipeline,
    atmo_pipeline: wgpu::RenderPipeline,
    depth_view: wgpu::TextureView,
    msaa_view: wgpu::TextureView,
    bind_group_layout: wgpu::BindGroupLayout,
    globe_vbuf: wgpu::Buffer,
    globe_ibuf: wgpu::Buffer,
    globe_idx_count: u32,
    atmo_vbuf: wgpu::Buffer,
    atmo_ibuf: wgpu::Buffer,
    atmo_idx_count: u32,
    entity_vbuf: wgpu::Buffer,
    entity_ibuf: wgpu::Buffer,
    entity_idx_count: u32,
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    camera: OrbitCamera,
    mouse_pressed: bool,
    last_mouse: [f32; 2],
    time: f32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            camera: OrbitCamera::new(),
            mouse_pressed: false,
            last_mouse: [0.0; 2],
            time: 0.0,
        }
    }

    fn create_msaa_texture(
        device: &wgpu::Device,
        w: u32,
        h: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("msaa"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_depth_texture(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("depth"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn init_gpu(&mut self, window: Arc<Window>) {
        use wgpu::util::DeviceExt;

        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12 | wgpu::Backends::METAL,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        println!(
            "GPU: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))
        .unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let make_pipeline =
            |cull: Option<wgpu::Face>, blend: Option<wgpu::BlendState>, depth_write: bool| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::layout()],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: cull,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: depth_write,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 4,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview: None,
                    cache: None,
                })
            };

        let globe_pipeline = make_pipeline(Some(wgpu::Face::Back), None, true);
        let atmo_pipeline = make_pipeline(
            Some(wgpu::Face::Front),
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
        );

        // Globe sphere (high res)
        let (gv, gi) = generate_sphere(96, 48, 1.0, [1.0, 1.0, 1.0]); // white = use procedural
        let globe_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&gv),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let globe_ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&gi),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Atmosphere shell (slightly larger, front-face culled for inner glow)
        let (av, ai) = generate_sphere(64, 32, 1.06, [0.3, 0.5, 1.0]);
        let atmo_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&av),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let atmo_ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&ai),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Entity markers (27K on surface)
        let (ev, ei) = generate_entity_markers(5000, 1.0);
        let entity_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&ev),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let entity_ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&ei),
            usage: wgpu::BufferUsages::INDEX,
        });

        let depth_view = Self::create_depth_texture(&device, config.width, config.height);
        let msaa_view = Self::create_msaa_texture(&device, config.width, config.height, format);

        self.gpu = Some(GpuState {
            surface,
            device,
            queue,
            config,
            globe_pipeline,
            atmo_pipeline,
            depth_view,
            msaa_view,
            bind_group_layout,
            globe_vbuf,
            globe_ibuf,
            globe_idx_count: gi.len() as u32,
            atmo_vbuf,
            atmo_ibuf,
            atmo_idx_count: ai.len() as u32,
            entity_vbuf,
            entity_ibuf,
            entity_idx_count: ei.len() as u32,
        });
    }

    fn render(&mut self) {
        let gpu = self.gpu.as_ref().unwrap();
        let output = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let aspect = gpu.config.width as f32 / gpu.config.height as f32;
        let cam_pos = self.camera.position();
        let vp = glam::Mat4::perspective_rh(50_f32.to_radians(), aspect, 0.01, 50.0)
            * glam::Mat4::look_at_rh(cam_pos, glam::Vec3::ZERO, glam::Vec3::Y);

        // Slowly rotating sun
        let sun_angle = self.time * 0.1;
        let light_dir = glam::Vec3::new(sun_angle.cos(), -0.3, sun_angle.sin()).normalize();

        let make_uniforms =
            |model: glam::Mat4, metallic: f32, roughness: f32, is_atmo: f32| -> Uniforms {
                Uniforms {
                    view_proj: vp.to_cols_array_2d(),
                    model: model.to_cols_array_2d(),
                    camera_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 0.0],
                    light_dir: [light_dir.x, light_dir.y, light_dir.z, 0.0],
                    params: [self.time, metallic, roughness, is_atmo],
                }
            };

        let make_bind_group = |uniforms: &Uniforms| -> wgpu::BindGroup {
            let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.queue
                .write_buffer(&buf, 0, bytemuck::bytes_of(uniforms));
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &gpu.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf.as_entire_binding(),
                }],
            })
        };

        // Slow globe rotation
        let globe_rot = glam::Mat4::from_rotation_y(self.time * 0.05);

        let globe_u = make_uniforms(globe_rot, 0.0, 0.65, 0.0);
        let globe_bg = make_bind_group(&globe_u);

        let entity_u = make_uniforms(globe_rot, 0.5, 0.3, 0.0);
        let entity_bg = make_bind_group(&entity_u);

        let atmo_u = make_uniforms(globe_rot, 0.0, 0.0, 1.0);
        let atmo_bg = make_bind_group(&atmo_u);

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gpu.msaa_view,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gpu.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // 1. Globe
            pass.set_pipeline(&gpu.globe_pipeline);
            pass.set_bind_group(0, &globe_bg, &[]);
            pass.set_vertex_buffer(0, gpu.globe_vbuf.slice(..));
            pass.set_index_buffer(gpu.globe_ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..gpu.globe_idx_count, 0, 0..1);

            // 2. Entity markers
            pass.set_bind_group(0, &entity_bg, &[]);
            pass.set_vertex_buffer(0, gpu.entity_vbuf.slice(..));
            pass.set_index_buffer(gpu.entity_ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..gpu.entity_idx_count, 0, 0..1);

            // 3. Atmosphere (transparent, front-face culled)
            pass.set_pipeline(&gpu.atmo_pipeline);
            pass.set_bind_group(0, &atmo_bg, &[]);
            pass.set_vertex_buffer(0, gpu.atmo_vbuf.slice(..));
            pass.set_index_buffer(gpu.atmo_ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..gpu.atmo_idx_count, 0, 0..1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title("Penumbra — Globe Demo")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
            let window = Arc::new(event_loop.create_window(attrs).unwrap());
            self.init_gpu(window.clone());
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                event_loop.exit()
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.config.width = size.width.max(1);
                    gpu.config.height = size.height.max(1);
                    gpu.surface.configure(&gpu.device, &gpu.config);
                    gpu.depth_view = Self::create_depth_texture(
                        &gpu.device,
                        gpu.config.width,
                        gpu.config.height,
                    );
                    gpu.msaa_view = Self::create_msaa_texture(
                        &gpu.device,
                        gpu.config.width,
                        gpu.config.height,
                        gpu.config.format,
                    );
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = state == ElementState::Pressed;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = [position.x as f32, position.y as f32];
                if self.mouse_pressed {
                    self.camera
                        .orbit(pos[0] - self.last_mouse[0], pos[1] - self.last_mouse[1]);
                }
                self.last_mouse = pos;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
                };
                self.camera.zoom(scroll);
            }
            WindowEvent::RedrawRequested => {
                self.time += 1.0 / 60.0;
                if !self.mouse_pressed {
                    self.camera.theta += 0.002;
                }
                self.render();
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    println!("Penumbra Globe Demo");
    println!("Controls: Left drag = orbit, Scroll = zoom, Escape = quit\n");
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut App::new()).unwrap();
}
