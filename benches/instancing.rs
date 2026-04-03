//! Benchmark: 27K entity instanced rendering throughput.
//!
//! Measures the time to:
//! - Generate 27K InstanceData entries
//! - Upload to an InstanceBatch
//! - CPU frustum cull 27K instances
//!
//! Run: `cargo bench --bench instancing`

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use glam::Mat4;
use penumbra_backend::MeshId;
use penumbra_instance::{InstanceBatchDesc, InstanceData, InstanceManager, cpu_frustum_cull};

fn generate_instances(count: usize) -> Vec<InstanceData> {
    (0..count)
        .map(|i| {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU * 5.0;
            let radius = 50.0 + (i as f32 / count as f32) * 200.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let y = (i as f32 * 0.37).sin() * 5.0;

            let mut transform = [0.0_f32; 16];
            let mat = Mat4::from_translation(glam::Vec3::new(x, y, z))
                * Mat4::from_scale(glam::Vec3::splat(0.3));
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

fn bench_instance_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_generation");
    for count in [1_000, 10_000, 27_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                black_box(generate_instances(count));
            });
        });
    }
    group.finish();
}

fn bench_batch_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_update");
    for count in [1_000, 10_000, 27_000] {
        let instances = generate_instances(count);
        let mut mgr = InstanceManager::new();
        let batch_id = mgr.create_batch(InstanceBatchDesc {
            mesh: MeshId(0),
            max_instances: 30_000,
            label: Some("bench".to_string()),
        });

        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, _| {
            b.iter(|| {
                mgr.update_batch(batch_id, black_box(instances.clone()))
                    .unwrap();
            });
        });
    }
    group.finish();
}

fn bench_cpu_frustum_cull(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_frustum_cull");
    for count in [1_000, 10_000, 27_000] {
        let instances = generate_instances(count);
        let view = Mat4::look_at_rh(
            glam::Vec3::new(0.0, 50.0, 100.0),
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );
        let proj = Mat4::perspective_rh(60_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
        let vp = proj * view;

        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, _| {
            b.iter(|| {
                black_box(cpu_frustum_cull(black_box(&instances), vp));
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_instance_generation,
    bench_batch_update,
    bench_cpu_frustum_cull
);
criterion_main!(benches);
