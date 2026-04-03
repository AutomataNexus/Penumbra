//! Quick GPU probe — check what adapter wgpu sees.
//! Run: cargo run --bin gpu_probe --release

fn main() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    println!("=== Available adapters ===");
    for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
        let info = adapter.get_info();
        println!("  {} ({:?}, {:?})", info.name, info.backend, info.device_type);
        let limits = adapter.limits();
        println!("    max_texture_size: {}", limits.max_texture_dimension_2d);
        println!("    max_buffer_size: {}", limits.max_buffer_size);
        let features = adapter.features();
        println!("    compute: {}", features.contains(wgpu::Features::empty()));
        println!("    timestamp_query: {}", features.contains(wgpu::Features::TIMESTAMP_QUERY));
    }

    // Try to get the high-performance adapter
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }));

    match adapter {
        Some(adapter) => {
            let info = adapter.get_info();
            println!("\n=== Selected adapter ===");
            println!("  {} ({:?})", info.name, info.backend);

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("gpu_probe"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            ))
            .expect("Failed to create device");

            println!("  Device created successfully");

            // Create a test texture to confirm GPU memory works
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("test"),
                size: wgpu::Extent3d { width: 4096, height: 4096, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            println!("  4096x4096 texture created on GPU");
            drop(tex);

            // Quick compute dispatch test
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("test_compute"),
                source: wgpu::ShaderSource::Wgsl(
                    "@compute @workgroup_size(64) fn main() {}".into(),
                ),
            });
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("test"),
                layout: None,
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, ..Default::default() });
                pass.set_pipeline(&pipeline);
                pass.dispatch_workgroups(1, 1, 1);
            }
            queue.submit(std::iter::once(encoder.finish()));
            device.poll(wgpu::Maintain::Wait);
            println!("  Compute dispatch successful");

            println!("\n  GPU is ready for benchmarks.");
        }
        None => {
            println!("\n  No high-performance adapter found.");
            println!("  Only fallback adapter available — GPU benchmarks won't be meaningful.");
        }
    }
}
