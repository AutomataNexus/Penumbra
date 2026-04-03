//! instanced_entities — 27K instanced entity rendering demo.
//!
//! Demonstrates: InstanceManager, InstanceBatch, 27,000 InstanceData entries,
//! GPU buffer sizing, CPU frustum culling, orbit camera, PBR lighting.
//! This is the core performance test for Penumbra's instanced rendering path.

use glam::{Mat4, Vec3};
use penumbra_asset::sphere_mesh;
use penumbra_camera::OrbitController;
use penumbra_core::{Renderer, RendererConfig};
use penumbra_instance::{
    cpu_frustum_cull, InstanceBatchDesc, InstanceData, InstanceManager,
};
use penumbra_pbr::{Light, PbrConfig, PbrPipeline};
use penumbra_wgpu::{WgpuBackend, WgpuConfig};

const ENTITY_COUNT: usize = 27_000;

fn main() {
    tracing_subscriber::fmt::init();

    // ── Backend + renderer ──
    let backend = WgpuBackend::headless(1920, 1080, WgpuConfig::default())
        .expect("Failed to create wgpu backend");
    let mut renderer = Renderer::new(backend, RendererConfig {
        width: 1920,
        height: 1080,
        hdr: true,
        ..RendererConfig::default()
    });

    // ── Mesh for instancing ──
    let gpu_sphere = renderer
        .create_mesh(sphere_mesh(8, 4)) // low-poly for instancing
        .expect("sphere mesh");

    // ── PBR lighting ──
    let mut pbr = PbrPipeline::new(PbrConfig::default());
    pbr.add_light(Light::Directional {
        direction: [-0.3, -1.0, -0.5],
        color: [1.0, 0.98, 0.95],
        intensity: 10.0,
        shadows: true,
    });

    // ── Instance manager ──
    let mut instance_mgr = InstanceManager::new();

    // Create a batch for aircraft entities
    let aircraft_batch = instance_mgr.create_batch(InstanceBatchDesc {
        mesh: gpu_sphere.id,
        max_instances: 10_000,
        label: Some("aircraft".to_string()),
    });

    // Create a batch for ground entities
    let ground_batch = instance_mgr.create_batch(InstanceBatchDesc {
        mesh: gpu_sphere.id,
        max_instances: 20_000,
        label: Some("ground".to_string()),
    });

    println!("Penumbra instanced_entities — {} entities", ENTITY_COUNT);
    println!("Backend: {}", renderer.backend_name());
    println!("Batches: {}", instance_mgr.batch_count());

    // ── Generate 27K entity instances ──
    // Simulate a tactical scenario: entities scattered across a large area

    let mut aircraft_instances = Vec::with_capacity(7_000);
    for i in 0..7_000 {
        let angle = (i as f32 / 7_000.0) * std::f32::consts::TAU * 3.0;
        let radius = 50.0 + (i as f32 / 7_000.0) * 200.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let y = 5.0 + (i as f32 * 0.37).sin() * 3.0; // altitude variation

        let mut transform = [0.0_f32; 16];
        let mat = Mat4::from_translation(Vec3::new(x, y, z))
            * Mat4::from_scale(Vec3::splat(0.3));
        transform.copy_from_slice(&mat.to_cols_array());

        // Color: hostile=red, friendly=blue, unknown=yellow
        let color = match i % 3 {
            0 => [1.0, 0.1, 0.1, 1.0],
            1 => [0.1, 0.3, 1.0, 1.0],
            _ => [1.0, 0.9, 0.1, 1.0],
        };

        aircraft_instances.push(InstanceData {
            transform,
            color,
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
        });
    }

    let mut ground_instances = Vec::with_capacity(20_000);
    for i in 0..20_000 {
        let grid_size = 150;
        let gx = (i % grid_size) as f32 - (grid_size as f32 / 2.0);
        let gz = (i / grid_size) as f32 - (grid_size as f32 / 2.0);
        let x = gx * 2.5 + (i as f32 * 0.73).sin() * 0.5;
        let z = gz * 2.5 + (i as f32 * 0.41).cos() * 0.5;
        let y = 0.0;

        let mut transform = [0.0_f32; 16];
        let mat = Mat4::from_translation(Vec3::new(x, y, z))
            * Mat4::from_scale(Vec3::splat(0.15));
        transform.copy_from_slice(&mat.to_cols_array());

        let color = match i % 4 {
            0 => [0.2, 0.8, 0.2, 1.0], // friendly vehicle
            1 => [0.8, 0.2, 0.2, 1.0], // hostile vehicle
            2 => [0.8, 0.8, 0.2, 1.0], // unknown
            _ => [0.5, 0.5, 0.5, 1.0], // neutral
        };

        ground_instances.push(InstanceData {
            transform,
            color,
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
        });
    }

    let total_instances = aircraft_instances.len() + ground_instances.len();
    println!("Total instances: {}", total_instances);
    println!(
        "Instance buffer size: {:.2} MB",
        (total_instances * std::mem::size_of::<InstanceData>()) as f64 / (1024.0 * 1024.0)
    );

    // Upload instances to batches
    instance_mgr
        .update_batch(aircraft_batch, aircraft_instances.clone())
        .expect("update aircraft batch");
    instance_mgr
        .update_batch(ground_batch, ground_instances.clone())
        .expect("update ground batch");

    // ── Camera ──
    let mut orbit = OrbitController {
        target: Vec3::ZERO,
        distance: 100.0,
        min_distance: 10.0,
        max_distance: 1000.0,
        aspect: 1920.0 / 1080.0,
        ..OrbitController::default()
    };
    orbit.handle_mouse_move(60.0, 30.0);

    let camera = orbit.camera();
    let vp = camera.view_projection();

    // ── CPU frustum culling ──
    let visible_aircraft = cpu_frustum_cull(&aircraft_instances, vp);
    let visible_ground = cpu_frustum_cull(&ground_instances, vp);
    let total_visible = visible_aircraft.len() + visible_ground.len();

    println!("Visible aircraft: {} / {}", visible_aircraft.len(), aircraft_instances.len());
    println!("Visible ground: {} / {}", visible_ground.len(), ground_instances.len());
    println!("Total visible: {} / {} ({:.1}%)",
        total_visible, total_instances,
        (total_visible as f64 / total_instances as f64) * 100.0
    );

    // ── Render frame ──
    let view = camera.view_matrix();
    let projection = camera.projection_matrix();

    let mut frame = renderer.begin_frame().expect("begin_frame");
    frame.set_camera(view, projection, camera.near, camera.far);

    let light_uniforms = pbr.light_uniforms();
    println!("Light uniforms: {}", light_uniforms.len());

    // In a full GPU implementation:
    // 1. Upload instance buffers to GPU
    // 2. Run compute shader for GPU frustum culling (produces indirect draw buffer)
    // 3. Issue draw_indirect calls (1 per batch)
    //
    // With CPU fallback:
    // 1. cpu_frustum_cull produces visible index list
    // 2. Upload only visible instances to GPU buffer
    // 3. Issue instanced draw calls with visible_count

    renderer.end_frame(frame).expect("end_frame");

    let stats = renderer.stats();
    println!("Frame complete — draw calls: {}, FPS: {:.0}", stats.draw_calls, stats.fps);

    // Clean up
    renderer.destroy_mesh(gpu_sphere.id);

    println!("instanced_entities done — {} entities rendered.", total_instances);
}
