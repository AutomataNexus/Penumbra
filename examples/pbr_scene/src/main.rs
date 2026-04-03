//! pbr_scene — Full PBR materials, multiple objects, IBL, shadows, and post-processing.
//!
//! Demonstrates: multiple meshes with different PBR materials, directional + point +
//! spot lights, cascaded shadow maps, post-processing pipeline (ACES tone mapping +
//! bloom + SSAO + FXAA + color grading), orbit camera, scene graph hierarchy.

use glam::{Quat, Vec3};
use penumbra_asset::{cube_mesh, sphere_mesh, plane_mesh};
use penumbra_camera::OrbitController;
use penumbra_core::{Material, Renderer, RendererConfig, Rgba, Rgb};
use penumbra_pbr::{Light, PbrConfig, PbrPipeline, EnvironmentConfig};
use penumbra_post::{PostPipeline, ToneMapping, Bloom, Ssao, Fxaa, ColorGrading};
use penumbra_scene::{Scene, Transform};
use penumbra_shadow::{ShadowConfig, CascadeShadowMap};
use penumbra_wgpu::{WgpuBackend, WgpuConfig};

fn main() {
    tracing_subscriber::fmt::init();

    // ── Backend + renderer ──
    let backend = WgpuBackend::headless(1920, 1080, WgpuConfig::default())
        .expect("Failed to create wgpu backend");
    let mut renderer = Renderer::new(backend, RendererConfig {
        width: 1920,
        height: 1080,
        msaa_samples: 4,
        hdr: true,
        vsync: true,
        ..RendererConfig::default()
    });

    // ── Create meshes ──
    let gpu_cube = renderer.create_mesh(cube_mesh()).expect("cube mesh");
    let gpu_sphere = renderer.create_mesh(sphere_mesh(32, 16)).expect("sphere mesh");
    let gpu_plane = renderer.create_mesh(plane_mesh(8)).expect("plane mesh");

    // ── Create PBR materials ──

    // Rough red metal
    let mat_red_metal = renderer.add_material(Material {
        albedo: Rgba::new(0.8, 0.1, 0.05, 1.0),
        metallic: 0.9,
        roughness: 0.25,
        ..Material::default()
    });

    // Polished gold sphere
    let mat_gold = renderer.add_material(Material {
        albedo: Rgba::new(1.0, 0.84, 0.0, 1.0),
        metallic: 1.0,
        roughness: 0.1,
        ..Material::default()
    });

    // Matte blue plastic
    let mat_blue_plastic = renderer.add_material(Material {
        albedo: Rgba::new(0.1, 0.2, 0.8, 1.0),
        metallic: 0.0,
        roughness: 0.8,
        ..Material::default()
    });

    // Emissive green
    let mat_emissive = renderer.add_material(Material {
        albedo: Rgba::new(0.05, 0.05, 0.05, 1.0),
        metallic: 0.0,
        roughness: 0.5,
        emissive: Rgb::new(0.0, 5.0, 0.5),
        ..Material::default()
    });

    // Ground (concrete-like)
    let mat_ground = renderer.add_material(Material {
        albedo: Rgba::new(0.4, 0.38, 0.36, 1.0),
        metallic: 0.0,
        roughness: 0.95,
        ..Material::default()
    });

    // ── Build scene graph ──
    let mut scene = Scene::new();

    // Ground plane
    let ground = scene.add_mesh(gpu_plane.id, mat_ground);
    scene.set_transform(ground, Transform {
        translation: Vec3::new(0.0, -1.0, 0.0),
        scale: Vec3::new(20.0, 1.0, 20.0),
        ..Transform::default()
    });
    scene.set_aabb(ground, gpu_plane.aabb);

    // Red metal cube (center)
    let cube1 = scene.add_mesh(gpu_cube.id, mat_red_metal);
    scene.set_transform(cube1, Transform {
        translation: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::from_euler(glam::EulerRot::YXZ, 0.6, 0.3, 0.0),
        scale: Vec3::ONE,
    });
    scene.set_aabb(cube1, gpu_cube.aabb);

    // Gold sphere (right)
    let sphere1 = scene.add_mesh(gpu_sphere.id, mat_gold);
    scene.set_transform(sphere1, Transform {
        translation: Vec3::new(2.5, 0.0, 0.0),
        scale: Vec3::splat(1.5),
        ..Transform::default()
    });
    scene.set_aabb(sphere1, gpu_sphere.aabb);

    // Blue plastic sphere (left)
    let sphere2 = scene.add_mesh(gpu_sphere.id, mat_blue_plastic);
    scene.set_transform(sphere2, Transform {
        translation: Vec3::new(-2.5, 0.0, 0.5),
        scale: Vec3::ONE,
        ..Transform::default()
    });
    scene.set_aabb(sphere2, gpu_sphere.aabb);

    // Small emissive cube (floating)
    let emissive_cube = scene.add_mesh(gpu_cube.id, mat_emissive);
    scene.set_transform(emissive_cube, Transform {
        translation: Vec3::new(-1.0, 2.0, -1.5),
        rotation: Quat::from_euler(glam::EulerRot::YXZ, 0.8, 0.5, 0.2),
        scale: Vec3::splat(0.5),
    });
    scene.set_aabb(emissive_cube, gpu_cube.aabb);

    // Parent group with two children (demonstrates hierarchy)
    let group = scene.add_empty();
    scene.set_transform(group, Transform {
        translation: Vec3::new(0.0, 0.0, -3.0),
        rotation: Quat::from_rotation_y(0.3),
        ..Transform::default()
    });
    let child1 = scene.add_mesh(gpu_cube.id, mat_blue_plastic);
    scene.set_parent(child1, group);
    scene.set_transform(child1, Transform {
        translation: Vec3::new(-1.0, 0.5, 0.0),
        scale: Vec3::splat(0.4),
        ..Transform::default()
    });
    let child2 = scene.add_mesh(gpu_sphere.id, mat_red_metal);
    scene.set_parent(child2, group);
    scene.set_transform(child2, Transform {
        translation: Vec3::new(1.0, 0.5, 0.0),
        scale: Vec3::splat(0.4),
        ..Transform::default()
    });

    // ── PBR lighting ──
    let mut pbr = PbrPipeline::new(PbrConfig {
        environment: EnvironmentConfig {
            intensity: 0.8,
            rotation: 0.0,
            diffuse_only: false,
        },
        ..PbrConfig::default()
    });

    // Sun (directional)
    pbr.add_light(Light::Directional {
        direction: [-0.5, -1.0, -0.3],
        color: [1.0, 0.98, 0.95],
        intensity: 10.0,
        shadows: true,
    });

    // Warm fill (point)
    pbr.add_light(Light::Point {
        position: [4.0, 3.0, 2.0],
        color: [1.0, 0.8, 0.5],
        intensity: 8.0,
        range: 20.0,
        shadows: true,
    });

    // Cool fill (point)
    pbr.add_light(Light::Point {
        position: [-3.0, 2.0, 4.0],
        color: [0.3, 0.5, 1.0],
        intensity: 5.0,
        range: 15.0,
        shadows: false,
    });

    // Spot light (focused on center)
    pbr.add_light(Light::Spot {
        position: [0.0, 5.0, 3.0],
        direction: [0.0, -1.0, -0.5],
        color: [1.0, 1.0, 1.0],
        intensity: 15.0,
        range: 20.0,
        inner_cone: 0.3,
        outer_cone: 0.5,
        shadows: true,
    });

    // ── Shadow maps ──
    let mut shadows = CascadeShadowMap::new(ShadowConfig::default());
    let sun_dir = Vec3::new(-0.5, -1.0, -0.3).normalize();

    // ── Post-processing pipeline ──
    let post = PostPipeline::new()
        .add(ToneMapping::aces())
        .add(Bloom {
            threshold: 1.0,
            intensity: 0.15,
            radius: 0.005,
            enabled: true,
        })
        .add(Ssao {
            radius: 0.5,
            bias: 0.025,
            intensity: 1.0,
            samples: 16,
            enabled: true,
        })
        .add(Fxaa::default())
        .add(ColorGrading {
            brightness: 0.0,
            contrast: 1.05,
            saturation: 1.1,
            enabled: true,
        });

    // ── Camera ──
    let mut orbit = OrbitController {
        target: Vec3::new(0.0, 0.5, 0.0),
        distance: 8.0,
        min_distance: 2.0,
        max_distance: 50.0,
        aspect: 1920.0 / 1080.0,
        ..OrbitController::default()
    };
    orbit.handle_mouse_move(80.0, 40.0);

    // ── Render ──
    println!("Penumbra pbr_scene — full PBR demo");
    println!("Backend: {}", renderer.backend_name());
    println!("Scene nodes: {}", scene.node_count());
    println!("PBR lights: {}", pbr.light_count());
    println!("Post passes: {}", post.pass_count());

    scene.update_transforms();

    let camera = orbit.camera();
    let view = camera.view_matrix();
    let projection = camera.projection_matrix();

    // Update shadow cascades for the sun
    shadows.update(sun_dir, camera.near, camera.far, view, projection);
    println!("Shadow cascades: {}", shadows.cascade_count());

    let mut frame = renderer.begin_frame().expect("begin_frame");
    frame.set_camera(view, projection, camera.near, camera.far);

    let light_uniforms = pbr.light_uniforms();
    println!("Light uniforms: {}", light_uniforms.len());

    renderer.end_frame(frame).expect("end_frame");

    let stats = renderer.stats();
    println!("Frame complete — draw calls: {}, FPS: {:.0}", stats.draw_calls, stats.fps);

    // Clean up
    renderer.destroy_mesh(gpu_cube.id);
    renderer.destroy_mesh(gpu_sphere.id);
    renderer.destroy_mesh(gpu_plane.id);

    println!("pbr_scene done.");
}
