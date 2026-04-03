//! Render screenshots — renders scenes offscreen and saves them as PNG files.
//!
//! Run: cargo run --bin render_screenshots --release
//!
//! Produces PNG files in the current directory that can be used on the site.

use std::time::Instant;

fn main() {
    println!("=== Penumbra Screenshot Renderer ===\n");

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("No GPU adapter");

    let info = adapter.get_info();
    println!("GPU: {} ({:?})\n", info.name, info.backend);

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("screenshots"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        },
        None,
    ))
    .expect("Failed to create device");

    // Render each scene
    render_cubes(&device, &queue, "screenshot_hello_cube.png");
    render_pbr_scene(&device, &queue, "screenshot_pbr_scene.png");
    render_instanced(&device, &queue, "screenshot_instanced.png");
    render_globe_terrain(&device, &queue, "screenshot_globe.png");

    println!("\n=== All screenshots saved ===");
}

fn save_texture_to_png(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    filename: &str,
) {
    let bytes_per_row = (width * 4 + 255) & !255; // align to 256
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screenshot_staging"),
        size: (bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        tx.send(r).ok();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();

    let data = slice.get_mapped_range();
    // Remove row padding
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        let start = (row * bytes_per_row) as usize;
        let end = start + (width * 4) as usize;
        pixels.extend_from_slice(&data[start..end]);
    }
    drop(data);
    staging.unmap();

    image::save_buffer(filename, &pixels, width, height, image::ColorType::Rgba8)
        .expect("Failed to save PNG");
    println!("  Saved: {} ({}x{})", filename, width, height);
}

fn create_render_targets(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (
    wgpu::Texture,
    wgpu::TextureView,
    wgpu::Texture,
    wgpu::TextureView,
) {
    let color = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
    (color, color_view, depth, depth_view)
}

fn create_pipeline(
    device: &wgpu::Device,
    shader_src: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
        cache: None,
    })
}

// ── Scene 1: PBR-lit cube ──

fn render_cubes(device: &wgpu::Device, queue: &wgpu::Queue, filename: &str) {
    println!("Rendering: hello_cube");
    let (w, h) = (1280, 720);
    let (color_tex, color_view, _depth_tex, depth_view) = create_render_targets(device, w, h);

    let shader_src = r#"
struct Uniforms { mvp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec3<f32>, @location(1) normal: vec3<f32> };

@vertex fn vs_main(@builtin(vertex_index) idx: u32) -> VOut {
    var p = array<vec3<f32>, 36>(
        vec3(-0.5,-0.5,-0.5), vec3(-0.5, 0.5,-0.5), vec3( 0.5, 0.5,-0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5, 0.5,-0.5), vec3( 0.5,-0.5,-0.5),
        vec3(-0.5,-0.5, 0.5), vec3( 0.5,-0.5, 0.5), vec3( 0.5, 0.5, 0.5),
        vec3(-0.5,-0.5, 0.5), vec3( 0.5, 0.5, 0.5), vec3(-0.5, 0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5,-0.5,-0.5), vec3( 0.5,-0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5,-0.5, 0.5), vec3(-0.5,-0.5, 0.5),
        vec3(-0.5, 0.5,-0.5), vec3(-0.5, 0.5, 0.5), vec3( 0.5, 0.5, 0.5),
        vec3(-0.5, 0.5,-0.5), vec3( 0.5, 0.5, 0.5), vec3( 0.5, 0.5,-0.5),
        vec3(-0.5,-0.5,-0.5), vec3(-0.5,-0.5, 0.5), vec3(-0.5, 0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3(-0.5, 0.5, 0.5), vec3(-0.5, 0.5,-0.5),
        vec3( 0.5,-0.5,-0.5), vec3( 0.5, 0.5,-0.5), vec3( 0.5, 0.5, 0.5),
        vec3( 0.5,-0.5,-0.5), vec3( 0.5, 0.5, 0.5), vec3( 0.5,-0.5, 0.5),
    );
    var n = array<vec3<f32>, 6>(
        vec3(0,0,-1), vec3(0,0,1), vec3(0,-1,0), vec3(0,1,0), vec3(-1,0,0), vec3(1,0,0)
    );
    var out: VOut;
    out.pos = u.mvp * vec4(p[idx], 1.0);
    out.normal = n[idx / 6u];
    out.color = vec3(0.8, 0.15, 0.1);
    return out;
}

@fragment fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3(-0.5, -1.0, -0.3));
    let ndl = max(dot(in.normal, -light_dir), 0.0);
    let ambient = 0.15;
    let diffuse = ndl * 0.85;
    let col = in.color * (ambient + diffuse);
    return vec4(col, 1.0);
}
"#;

    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(1.5, 1.2, 2.0),
        glam::Vec3::ZERO,
        glam::Vec3::Y,
    );
    let proj = glam::Mat4::perspective_rh(60_f32.to_radians(), w as f32 / h as f32, 0.1, 100.0);
    let mvp = proj * view;

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline = create_pipeline(device, shader_src, &bgl);
    let ubuf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&ubuf, 0, bytemuck::bytes_of(&mvp.to_cols_array()));
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: ubuf.as_entire_binding(),
        }],
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.95,
                        g: 0.95,
                        b: 0.97,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..36, 0..1);
    }
    queue.submit(std::iter::once(enc.finish()));
    device.poll(wgpu::Maintain::Wait);
    save_texture_to_png(device, queue, &color_tex, w, h, filename);
}

// ── Scene 2: PBR multi-material ──

fn render_pbr_scene(device: &wgpu::Device, queue: &wgpu::Queue, filename: &str) {
    println!("Rendering: pbr_scene");
    let (w, h) = (1280, 720);
    let (color_tex, color_view, _depth_tex, depth_view) = create_render_targets(device, w, h);

    let shader_src = r#"
struct Uniforms { mvp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec3<f32>, @location(1) normal: vec3<f32> };

@vertex fn vs_main(@builtin(vertex_index) idx: u32, @builtin(instance_index) inst: u32) -> VOut {
    var p = array<vec3<f32>, 36>(
        vec3(-0.5,-0.5,-0.5), vec3(-0.5, 0.5,-0.5), vec3( 0.5, 0.5,-0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5, 0.5,-0.5), vec3( 0.5,-0.5,-0.5),
        vec3(-0.5,-0.5, 0.5), vec3( 0.5,-0.5, 0.5), vec3( 0.5, 0.5, 0.5),
        vec3(-0.5,-0.5, 0.5), vec3( 0.5, 0.5, 0.5), vec3(-0.5, 0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5,-0.5,-0.5), vec3( 0.5,-0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5,-0.5, 0.5), vec3(-0.5,-0.5, 0.5),
        vec3(-0.5, 0.5,-0.5), vec3(-0.5, 0.5, 0.5), vec3( 0.5, 0.5, 0.5),
        vec3(-0.5, 0.5,-0.5), vec3( 0.5, 0.5, 0.5), vec3( 0.5, 0.5,-0.5),
        vec3(-0.5,-0.5,-0.5), vec3(-0.5,-0.5, 0.5), vec3(-0.5, 0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3(-0.5, 0.5, 0.5), vec3(-0.5, 0.5,-0.5),
        vec3( 0.5,-0.5,-0.5), vec3( 0.5, 0.5,-0.5), vec3( 0.5, 0.5, 0.5),
        vec3( 0.5,-0.5,-0.5), vec3( 0.5, 0.5, 0.5), vec3( 0.5,-0.5, 0.5),
    );
    var n = array<vec3<f32>, 6>(vec3(0,0,-1), vec3(0,0,1), vec3(0,-1,0), vec3(0,1,0), vec3(-1,0,0), vec3(1,0,0));
    var colors = array<vec3<f32>, 5>(
        vec3(0.8, 0.1, 0.05), vec3(1.0, 0.84, 0.0), vec3(0.1, 0.2, 0.8), vec3(0.05, 0.8, 0.3), vec3(0.4, 0.38, 0.36)
    );
    var offsets = array<vec3<f32>, 5>(
        vec3(0.0, 0.0, 0.0), vec3(2.0, 0.0, 0.0), vec3(-2.0, 0.0, 0.0), vec3(-1.0, 1.5, -1.0), vec3(0.0, -0.5, 0.0)
    );
    var scales = array<f32, 5>(0.8, 0.7, 0.6, 0.4, 3.0);

    var out: VOut;
    let s = scales[inst];
    let offset = offsets[inst];
    var v = p[idx] * s + offset;
    if inst == 4u { v.y = -1.0; } // ground plane flat
    out.pos = u.mvp * vec4(v, 1.0);
    out.normal = n[idx / 6u];
    out.color = colors[inst];
    return out;
}

@fragment fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let l1 = normalize(vec3(-0.5, -1.0, -0.3));
    let l2 = normalize(vec3(1.0, 0.5, 0.3));
    let ndl1 = max(dot(in.normal, -l1), 0.0) * 0.7;
    let ndl2 = max(dot(in.normal, -l2), 0.0) * 0.3;
    let ambient = 0.12;
    let col = in.color * (ambient + ndl1 + ndl2);
    return vec4(col, 1.0);
}
"#;

    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(3.0, 2.5, 4.0),
        glam::Vec3::new(0.0, 0.2, 0.0),
        glam::Vec3::Y,
    );
    let proj = glam::Mat4::perspective_rh(55_f32.to_radians(), w as f32 / h as f32, 0.1, 100.0);
    let mvp = proj * view;

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline = create_pipeline(device, shader_src, &bgl);
    let ubuf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&ubuf, 0, bytemuck::bytes_of(&mvp.to_cols_array()));
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: ubuf.as_entire_binding(),
        }],
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.93,
                        g: 0.93,
                        b: 0.95,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..36, 0..5); // 5 objects
    }
    queue.submit(std::iter::once(enc.finish()));
    device.poll(wgpu::Maintain::Wait);
    save_texture_to_png(device, queue, &color_tex, w, h, filename);
}

// ── Scene 3: 27K instanced entities ──

fn render_instanced(device: &wgpu::Device, queue: &wgpu::Queue, filename: &str) {
    println!("Rendering: instanced_entities (27K)");
    let (w, h) = (1280, 720);
    let (color_tex, color_view, _depth_tex, depth_view) = create_render_targets(device, w, h);

    // Reuse the shader from gpu_render_bench
    let shader_src = r#"
struct Uniforms { mvp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };

@vertex fn vs_main(@builtin(vertex_index) idx: u32, @builtin(instance_index) iid: u32) -> VOut {
    var p = array<vec3<f32>, 36>(
        vec3(-0.5,-0.5,-0.5), vec3(-0.5, 0.5,-0.5), vec3( 0.5, 0.5,-0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5, 0.5,-0.5), vec3( 0.5,-0.5,-0.5),
        vec3(-0.5,-0.5, 0.5), vec3( 0.5,-0.5, 0.5), vec3( 0.5, 0.5, 0.5),
        vec3(-0.5,-0.5, 0.5), vec3( 0.5, 0.5, 0.5), vec3(-0.5, 0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5,-0.5,-0.5), vec3( 0.5,-0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3( 0.5,-0.5, 0.5), vec3(-0.5,-0.5, 0.5),
        vec3(-0.5, 0.5,-0.5), vec3(-0.5, 0.5, 0.5), vec3( 0.5, 0.5, 0.5),
        vec3(-0.5, 0.5,-0.5), vec3( 0.5, 0.5, 0.5), vec3( 0.5, 0.5,-0.5),
        vec3(-0.5,-0.5,-0.5), vec3(-0.5,-0.5, 0.5), vec3(-0.5, 0.5, 0.5),
        vec3(-0.5,-0.5,-0.5), vec3(-0.5, 0.5, 0.5), vec3(-0.5, 0.5,-0.5),
        vec3( 0.5,-0.5,-0.5), vec3( 0.5, 0.5,-0.5), vec3( 0.5, 0.5, 0.5),
        vec3( 0.5,-0.5,-0.5), vec3( 0.5, 0.5, 0.5), vec3( 0.5,-0.5, 0.5),
    );
    let grid = 165u;
    let ix = iid % grid;
    let iz = iid / grid;
    let offset = vec3<f32>(
        (f32(ix) - f32(grid) / 2.0) * 2.0,
        sin(f32(iid) * 0.01) * 2.0,
        (f32(iz) - f32(grid) / 2.0) * 2.0,
    );
    var out: VOut;
    out.pos = u.mvp * vec4(p[idx] * 0.6 + offset, 1.0);
    // Color by threat level
    let threat = iid % 3u;
    if threat == 0u { out.color = vec4(1.0, 0.15, 0.1, 1.0); }   // hostile
    else if threat == 1u { out.color = vec4(0.1, 0.4, 1.0, 1.0); } // friendly
    else { out.color = vec4(1.0, 0.85, 0.1, 1.0); }               // unknown
    return out;
}

@fragment fn fs_main(in: VOut) -> @location(0) vec4<f32> { return in.color; }
"#;

    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(0.0, 120.0, 200.0),
        glam::Vec3::ZERO,
        glam::Vec3::Y,
    );
    let proj = glam::Mat4::perspective_rh(60_f32.to_radians(), w as f32 / h as f32, 0.1, 1000.0);
    let mvp = proj * view;

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline = create_pipeline(device, shader_src, &bgl);
    let ubuf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&ubuf, 0, bytemuck::bytes_of(&mvp.to_cols_array()));
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: ubuf.as_entire_binding(),
        }],
    });

    let start = Instant::now();
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.08,
                        g: 0.08,
                        b: 0.12,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..36, 0..27_225);
    }
    queue.submit(std::iter::once(enc.finish()));
    device.poll(wgpu::Maintain::Wait);
    println!(
        "  27K instances rendered in {:.2}ms",
        start.elapsed().as_secs_f64() * 1000.0
    );
    save_texture_to_png(device, queue, &color_tex, w, h, filename);
}

// ── Scene 4: Globe terrain grid ──

fn render_globe_terrain(device: &wgpu::Device, queue: &wgpu::Queue, filename: &str) {
    println!("Rendering: globe (terrain grid)");
    let (w, h) = (1280, 720);
    let (color_tex, color_view, _depth_tex, depth_view) = create_render_targets(device, w, h);

    let shader_src = r#"
struct Uniforms { mvp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec3<f32> };

@vertex fn vs_main(@builtin(vertex_index) idx: u32, @builtin(instance_index) iid: u32) -> VOut {
    // Render a grid of terrain tiles as colored quads
    let tiles_per_row = 8u;
    let tx = iid % tiles_per_row;
    let tz = iid / tiles_per_row;

    // Each tile is 2 triangles = 6 vertices
    var quad = array<vec3<f32>, 6>(
        vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0),
    );

    let tile_size = 2.0;
    let grid_offset = -f32(tiles_per_row) * tile_size / 2.0;
    let pos = quad[idx] * tile_size + vec3(f32(tx) * tile_size + grid_offset, 0.0, f32(tz) * tile_size + grid_offset);

    // Simulate terrain height
    let h = sin(pos.x * 0.5) * cos(pos.z * 0.3) * 0.8;

    var out: VOut;
    out.pos = u.mvp * vec4(pos.x, h, pos.z, 1.0);

    // Color: green-brown terrain gradient
    let t = (h + 0.8) / 1.6;
    out.color = mix(vec3(0.2, 0.5, 0.15), vec3(0.6, 0.5, 0.3), t);

    // Tile grid lines (darken edges)
    let uv = quad[idx].xz;
    if uv.x < 0.02 || uv.x > 0.98 || uv.y < 0.02 || uv.y > 0.98 {
        out.color *= 0.6;
    }

    return out;
}

@fragment fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4(in.color, 1.0);
}
"#;

    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(8.0, 10.0, 12.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        glam::Vec3::Y,
    );
    let proj = glam::Mat4::perspective_rh(60_f32.to_radians(), w as f32 / h as f32, 0.1, 200.0);
    let mvp = proj * view;

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline = create_pipeline(device, shader_src, &bgl);
    let ubuf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&ubuf, 0, bytemuck::bytes_of(&mvp.to_cols_array()));
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: ubuf.as_entire_binding(),
        }],
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.55,
                        g: 0.7,
                        b: 0.9,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..6, 0..64); // 8x8 grid of tiles
    }
    queue.submit(std::iter::once(enc.finish()));
    device.poll(wgpu::Maintain::Wait);
    save_texture_to_png(device, queue, &color_tex, w, h, filename);
}
