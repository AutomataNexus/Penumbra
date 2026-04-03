//! Benchmark: full tactical globe scene composite.
//!
//! Measures the complete frame preparation pipeline:
//! - Scene graph update (transform propagation)
//! - Instance data generation for 27K entities
//! - CPU frustum culling
//! - Light uniform generation
//! - Text layout
//! - Total frame orchestration time
//!
//! Run: `cargo bench --bench full_scene`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::{Mat4, Quat, Vec3};
use penumbra_camera::OrbitController;
use penumbra_instance::{cpu_frustum_cull, InstanceData};
use penumbra_pbr::{Light, PbrConfig, PbrPipeline};
use penumbra_scene::{Scene, Transform};
use penumbra_text::{FontAtlas, FontId, GlyphMetrics, TextBatch, layout_text};
use penumbra_backend::MeshId;
use penumbra_core::MaterialId;

fn build_scene(node_count: usize) -> Scene {
    let mut scene = Scene::new();
    for i in 0..node_count {
        let node = scene.add_mesh(MeshId(1), MaterialId(1));
        scene.set_transform(
            node,
            Transform {
                translation: Vec3::new(
                    (i as f32 * 1.7).cos() * 50.0,
                    (i as f32 * 0.3).sin() * 5.0,
                    (i as f32 * 1.7).sin() * 50.0,
                ),
                rotation: Quat::from_rotation_y(i as f32 * 0.1),
                scale: Vec3::ONE,
            },
        );
    }
    scene
}

fn generate_27k_instances() -> Vec<InstanceData> {
    (0..27_000)
        .map(|i| {
            let angle = (i as f32 / 27_000.0) * std::f32::consts::TAU * 5.0;
            let radius = 50.0 + (i as f32 / 27_000.0) * 200.0;
            let mut transform = [0.0_f32; 16];
            let mat = Mat4::from_translation(Vec3::new(
                angle.cos() * radius,
                (i as f32 * 0.37).sin() * 5.0,
                angle.sin() * radius,
            ));
            transform.copy_from_slice(&mat.to_cols_array());
            InstanceData {
                transform,
                color: [1.0, 0.0, 0.0, 1.0],
                uv_offset: [0.0, 0.0],
                uv_scale: [1.0, 1.0],
            }
        })
        .collect()
}

fn bench_scene_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_update");

    for count in [100, 1_000, 5_000] {
        let mut scene = build_scene(count);
        group.bench_function(format!("{count}_nodes"), |b| {
            b.iter(|| {
                scene.update_transforms();
                black_box(&scene);
            });
        });
    }

    group.finish();
}

fn bench_full_frame_prep(c: &mut Criterion) {
    c.bench_function("full_frame_27k", |b| {
        let instances = generate_27k_instances();
        let orbit = OrbitController {
            target: Vec3::ZERO,
            distance: 200.0,
            aspect: 16.0 / 9.0,
            ..OrbitController::default()
        };
        let camera = orbit.camera();
        let vp = camera.view_projection();

        let mut pbr = PbrPipeline::new(PbrConfig::default());
        pbr.add_light(Light::Directional {
            direction: [-0.5, -1.0, -0.3],
            color: [1.0, 0.98, 0.95],
            intensity: 10.0,
            shadows: true,
        });
        pbr.add_light(Light::Point {
            position: [4.0, 3.0, 2.0],
            color: [1.0, 0.8, 0.5],
            intensity: 8.0,
            range: 20.0,
            shadows: false,
        });

        let mut font_atlas = FontAtlas::new(FontId(0), 512, 512);
        for ch in "ENTITIES: 27000".chars() {
            font_atlas.add_glyph(GlyphMetrics {
                codepoint: ch,
                advance: 8.0,
                bearing_x: 0.0,
                bearing_y: 10.0,
                width: 8.0,
                height: 12.0,
                uv_min: [0.0, 0.0],
                uv_max: [0.01, 0.02],
            });
        }

        b.iter(|| {
            // 1. CPU frustum cull 27K instances
            let visible = cpu_frustum_cull(black_box(&instances), vp);

            // 2. Generate light uniforms
            let light_uniforms = pbr.light_uniforms();

            // 3. Layout HUD text
            let layout = layout_text(&font_atlas, "ENTITIES: 27000", 14.0);
            let mut text_batch = TextBatch::new();
            text_batch.add_layout(&layout, 0.0, [1.0, 1.0, 1.0, 1.0]);

            black_box((&visible, &light_uniforms, &text_batch));
        });
    });
}

fn bench_frustum_cull_27k(c: &mut Criterion) {
    let instances = generate_27k_instances();
    let orbit = OrbitController {
        distance: 200.0,
        ..OrbitController::default()
    };
    let vp = orbit.camera().view_projection();

    c.bench_function("cull_27k_instances", |b| {
        b.iter(|| {
            black_box(cpu_frustum_cull(black_box(&instances), vp));
        });
    });
}

criterion_group!(
    benches,
    bench_scene_update,
    bench_full_frame_prep,
    bench_frustum_cull_27k
);
criterion_main!(benches);
