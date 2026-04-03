//! GPU Render Benchmark — measures actual frame times through the full Vulkan pipeline.
//!
//! Creates a real wgpu surface, renders geometry through real render passes,
//! and measures wall-clock time per frame including GPU work.
//!
//! Run: cargo run --bin gpu_render_bench --release

use std::time::{Duration, Instant};

fn main() {
    println!("=== Penumbra GPU Render Benchmark ===");
    println!("Measuring actual GPU frame times through Vulkan pipeline.\n");

    // ── Create GPU device ──
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("No GPU adapter found");

    let info = adapter.get_info();
    println!("GPU: {} ({:?})", info.name, info.backend);

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("bench"),
            required_features: wgpu::Features::TIMESTAMP_QUERY,
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        },
        None,
    ))
    .expect("Failed to create device");

    // ── Create offscreen render target (1920x1080) ──
    let width = 1920u32;
    let height = 1080u32;
    let format = wgpu::TextureFormat::Rgba8Unorm;

    let color_target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("color_target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color_target.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_target.create_view(&wgpu::TextureViewDescriptor::default());

    // ── Create a simple render pipeline ──
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("bench_shader"),
        source: wgpu::ShaderSource::Wgsl(r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Uniforms {
    mvp: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32, @builtin(instance_index) instance_id: u32) -> VertexOutput {
    // Simple cube vertices inline
    var positions = array<vec3<f32>, 36>(
        vec3(-0.5, -0.5, -0.5), vec3(-0.5,  0.5, -0.5), vec3( 0.5,  0.5, -0.5),
        vec3(-0.5, -0.5, -0.5), vec3( 0.5,  0.5, -0.5), vec3( 0.5, -0.5, -0.5),
        vec3(-0.5, -0.5,  0.5), vec3( 0.5, -0.5,  0.5), vec3( 0.5,  0.5,  0.5),
        vec3(-0.5, -0.5,  0.5), vec3( 0.5,  0.5,  0.5), vec3(-0.5,  0.5,  0.5),
        vec3(-0.5, -0.5, -0.5), vec3( 0.5, -0.5, -0.5), vec3( 0.5, -0.5,  0.5),
        vec3(-0.5, -0.5, -0.5), vec3( 0.5, -0.5,  0.5), vec3(-0.5, -0.5,  0.5),
        vec3(-0.5,  0.5, -0.5), vec3(-0.5,  0.5,  0.5), vec3( 0.5,  0.5,  0.5),
        vec3(-0.5,  0.5, -0.5), vec3( 0.5,  0.5,  0.5), vec3( 0.5,  0.5, -0.5),
        vec3(-0.5, -0.5, -0.5), vec3(-0.5, -0.5,  0.5), vec3(-0.5,  0.5,  0.5),
        vec3(-0.5, -0.5, -0.5), vec3(-0.5,  0.5,  0.5), vec3(-0.5,  0.5, -0.5),
        vec3( 0.5, -0.5, -0.5), vec3( 0.5,  0.5, -0.5), vec3( 0.5,  0.5,  0.5),
        vec3( 0.5, -0.5, -0.5), vec3( 0.5,  0.5,  0.5), vec3( 0.5, -0.5,  0.5),
    );

    let pos = positions[idx];

    // Scatter instances in a grid
    let grid_size = 165u; // 165 * 165 = 27225 instances
    let ix = instance_id % grid_size;
    let iz = instance_id / grid_size;
    let offset = vec3<f32>(
        (f32(ix) - f32(grid_size) / 2.0) * 2.0,
        0.0,
        (f32(iz) - f32(grid_size) / 2.0) * 2.0,
    );

    var out: VertexOutput;
    out.position = uniforms.mvp * vec4(pos * 0.8 + offset, 1.0);

    // Color by instance
    let r = f32(ix) / f32(grid_size);
    let b = f32(iz) / f32(grid_size);
    out.color = vec4(r, 0.3, b, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bench_bgl"),
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("bench_layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("bench_pipeline"),
        layout: Some(&pipeline_layout),
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
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
        cache: None,
    });

    // ── Create uniform buffer with MVP matrix ──
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(0.0, 100.0, 200.0),
        glam::Vec3::ZERO,
        glam::Vec3::Y,
    );
    let proj = glam::Mat4::perspective_rh(
        60_f32.to_radians(),
        width as f32 / height as f32,
        0.1,
        1000.0,
    );
    let mvp = proj * view;

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniform"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&mvp.to_cols_array()));

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bench_bg"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let ctx = GpuCtx {
        device: &device,
        queue: &queue,
        color_view: &color_view,
        depth_view: &depth_view,
        pipeline: &pipeline,
        bind_group: &bind_group,
    };

    // ── Warmup (5 frames) ──
    println!("Warming up...");
    for _ in 0..5 {
        render_frame(&ctx, 27_225);
    }

    // ── Benchmark scenarios ──

    // Scenario 1: 27K instanced entities
    println!("\n--- Scenario 1: 27K instanced entities (1920x1080, depth + color) ---");
    let times = bench_frames(&ctx, 27_225, 200);
    print_stats("27K entities", &times);

    // Scenario 2: 10K instanced entities
    println!("\n--- Scenario 2: 10K instanced entities ---");
    let times = bench_frames(&ctx, 10_000, 200);
    print_stats("10K entities", &times);

    // Scenario 3: 1K instanced entities
    println!("\n--- Scenario 3: 1K instanced entities ---");
    let times = bench_frames(&ctx, 1_000, 200);
    print_stats("1K entities", &times);

    // Scenario 4: 50K instanced entities (stress test)
    println!("\n--- Scenario 4: 50K instanced entities (stress test) ---");
    let times = bench_frames(&ctx, 50_000, 100);
    print_stats("50K entities", &times);

    // Scenario 5: Empty frame (baseline)
    println!("\n--- Scenario 5: Empty frame (baseline) ---");
    let times = bench_frames(&ctx, 0, 200);
    print_stats("Empty frame", &times);

    println!("\n=== Benchmark complete ===");
    println!("Target: < 8ms for 27K entities at 60fps");
    println!("If 27K median < 8ms, the 60fps target is met.");
}

struct GpuCtx<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    color_view: &'a wgpu::TextureView,
    depth_view: &'a wgpu::TextureView,
    pipeline: &'a wgpu::RenderPipeline,
    bind_group: &'a wgpu::BindGroup,
}

fn render_frame(ctx: &GpuCtx, instance_count: u32) {
    let device = ctx.device;
    let queue = ctx.queue;
    let color_view = ctx.color_view;
    let depth_view = ctx.depth_view;
    let pipeline = ctx.pipeline;
    let bind_group = ctx.bind_group;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("frame"),
    });

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("main_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.15, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        if instance_count > 0 {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..36, 0..instance_count); // 36 verts per cube, N instances
        }
    }

    queue.submit(std::iter::once(encoder.finish()));
    device.poll(wgpu::Maintain::Wait); // Wait for GPU to finish
}

fn bench_frames(ctx: &GpuCtx, instance_count: u32, num_frames: usize) -> Vec<Duration> {
    let mut times = Vec::with_capacity(num_frames);
    for _ in 0..num_frames {
        let start = Instant::now();
        render_frame(ctx, instance_count);
        times.push(start.elapsed());
    }
    times
}

fn print_stats(label: &str, times: &[Duration]) {
    let mut sorted: Vec<f64> = times.iter().map(|t| t.as_secs_f64() * 1000.0).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let median = sorted[sorted.len() / 2];
    let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];
    let avg: f64 = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let fps = 1000.0 / median;
    let triangles = if label.contains("27K") { 27_225 * 12 } else if label.contains("10K") { 10_000 * 12 } else if label.contains("50K") { 50_000 * 12 } else if label.contains("1K") { 1_000 * 12 } else { 0 };

    println!("  {}", label);
    println!("    min:    {:.3} ms", min);
    println!("    median: {:.3} ms", median);
    println!("    avg:    {:.3} ms", avg);
    println!("    p99:    {:.3} ms", p99);
    println!("    max:    {:.3} ms", max);
    println!("    fps:    {:.0} (at median)", fps);
    if triangles > 0 {
        println!("    triangles: {} ({:.1}M)", triangles, triangles as f64 / 1_000_000.0);
    }
    let pass = if median < 8.0 { "PASS" } else { "FAIL" };
    println!("    < 8ms target: {}", pass);
}
