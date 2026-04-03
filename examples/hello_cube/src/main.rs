//! hello_cube — Simplest Penumbra 3D example.
//!
//! Renders a PBR-lit cube using the scene graph, orbit camera, and wgpu backend.
//! Demonstrates: backend init, mesh creation, material setup, PBR lighting,
//! scene graph hierarchy, camera system, and the frame loop.

use glam::{Quat, Vec3};
use penumbra_asset::cube_mesh;
use penumbra_camera::OrbitController;
use penumbra_core::{Material, Renderer, RendererConfig, Rgb, Rgba};
use penumbra_pbr::{Light, PbrConfig, PbrPipeline};
use penumbra_scene::{Scene, Transform};
use penumbra_wgpu::{WgpuBackend, WgpuConfig};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create headless wgpu backend (swap for window-based in a real app)
    let backend = WgpuBackend::headless(1280, 720, WgpuConfig::default())
        .expect("Failed to create wgpu backend");
    let mut renderer = Renderer::new(
        backend,
        RendererConfig {
            width: 1280,
            height: 720,
            msaa_samples: 4,
            hdr: true,
            vsync: true,
            ..RendererConfig::default()
        },
    );

    // ── Create the cube mesh on the GPU ──
    let cube_desc = cube_mesh();
    let gpu_mesh = renderer
        .create_mesh(cube_desc)
        .expect("Failed to create cube mesh");

    // ── Create a PBR material ──
    // Rough red metal — not a flat color cube
    let material_id = renderer.add_material(Material {
        albedo: Rgba::new(0.8, 0.1, 0.1, 1.0),
        metallic: 0.7,
        roughness: 0.35,
        emissive: Rgb::new(0.0, 0.0, 0.0),
        double_sided: false,
        ..Material::default()
    });

    // ── Set up the PBR lighting pipeline ──
    let mut pbr = PbrPipeline::new(PbrConfig::default());

    // Key light (sun)
    pbr.add_light(Light::Directional {
        direction: [-0.5, -1.0, -0.3],
        color: [1.0, 0.98, 0.95],
        intensity: 10.0,
        shadows: true,
    });

    // Fill light
    pbr.add_light(Light::Point {
        position: [3.0, 2.0, 4.0],
        color: [0.4, 0.5, 0.8],
        intensity: 5.0,
        range: 20.0,
        shadows: false,
    });

    // Rim light
    pbr.add_light(Light::Point {
        position: [-2.0, 1.0, -3.0],
        color: [0.9, 0.7, 0.3],
        intensity: 3.0,
        range: 15.0,
        shadows: false,
    });

    // ── Build the scene graph ──
    let mut scene = Scene::new();

    // Add the cube mesh to the scene
    let cube_node = scene.add_mesh(gpu_mesh.id, material_id);
    scene.set_aabb(cube_node, gpu_mesh.aabb);

    // Rotate the cube slightly so we see three faces
    scene.set_transform(
        cube_node,
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::from_euler(glam::EulerRot::YXZ, 0.4, 0.3, 0.0),
            scale: Vec3::ONE,
        },
    );

    // Add a ground plane node (empty — would have a plane mesh in a full example)
    let _ground = scene.add_empty();
    scene.set_transform(
        _ground,
        Transform {
            translation: Vec3::new(0.0, -1.0, 0.0),
            scale: Vec3::new(10.0, 0.01, 10.0),
            ..Transform::default()
        },
    );

    // ── Set up the orbit camera ──
    let mut orbit = OrbitController {
        target: Vec3::ZERO,
        distance: 4.0,
        min_distance: 1.0,
        max_distance: 50.0,
        sensitivity: 0.005,
        ..OrbitController::default()
    };

    // Angle the camera slightly
    orbit.handle_mouse_move(100.0, 60.0);

    // ── Render loop (headless: single frame) ──
    // In a real app this would be driven by winit's event loop
    println!("Penumbra hello_cube — rendering a PBR-lit cube");
    println!("Backend: {}", renderer.backend_name());
    println!("Scene nodes: {}", scene.node_count());
    println!("PBR lights: {}", pbr.light_count());

    // Propagate world transforms
    scene.update_transforms();

    // Get camera matrices
    let camera = orbit.camera();
    let view = camera.view_matrix();
    let projection = camera.projection_matrix();

    // Begin frame
    let mut frame = renderer.begin_frame().expect("begin_frame");
    frame.set_camera(view, projection, 0.1, 1000.0);

    // In a full implementation, scene.render() would generate draw calls and
    // submit them to the frame. Here we demonstrate the data flow:
    let light_uniforms = pbr.light_uniforms();
    println!("Light uniforms generated: {}", light_uniforms.len());
    println!("Frame time: {:.2}ms", frame.time);
    println!("Viewport: {}x{}", frame.width, frame.height);

    // End frame
    renderer.end_frame(frame).expect("end_frame");

    let stats = renderer.stats();
    println!(
        "Frame complete — draw calls: {}, FPS: {:.0}",
        stats.draw_calls, stats.fps
    );

    // Clean up
    renderer.destroy_mesh(gpu_mesh.id);

    println!("hello_cube done.");
}
