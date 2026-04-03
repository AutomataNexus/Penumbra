//! Benchmark: tile streaming and terrain mesh generation.
//!
//! Measures:
//! - Tile cache insert/lookup performance
//! - Terrain mesh generation at various resolutions
//! - Terrain-RGB decoding throughput
//!
//! Run: `cargo bench --bench tile_streaming`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use penumbra_terrain::{
    TileCache, TileCoord, TileData, TerrainData, decode_terrain_rgb, generate_tile_mesh,
};

fn bench_tile_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("tile_cache");

    group.bench_function("insert_256_tiles", |b| {
        b.iter(|| {
            let mut cache = TileCache::new(256);
            for i in 0..256_u32 {
                cache.insert(
                    TileCoord::new(i % 16, i / 16, 4),
                    TileData::Terrain(TerrainData {
                        heights: vec![0.0; 33 * 33],
                        width: 33,
                        height: 33,
                    }),
                );
            }
            black_box(&cache);
        });
    });

    group.bench_function("lookup_hit", |b| {
        let mut cache = TileCache::new(256);
        for i in 0..256_u32 {
            cache.insert(
                TileCoord::new(i % 16, i / 16, 4),
                TileData::Terrain(TerrainData {
                    heights: vec![0.0; 33 * 33],
                    width: 33,
                    height: 33,
                }),
            );
        }
        let coord = TileCoord::new(8, 8, 4);
        b.iter(|| {
            black_box(cache.get(&coord));
        });
    });

    group.bench_function("lru_eviction_pressure", |b| {
        let mut cache = TileCache::new(64);
        // Pre-fill
        for i in 0..64_u32 {
            cache.insert(
                TileCoord::new(i, 0, 0),
                TileData::Terrain(TerrainData {
                    heights: vec![0.0; 33 * 33],
                    width: 33,
                    height: 33,
                }),
            );
        }
        let mut counter = 64_u32;
        b.iter(|| {
            cache.insert(
                TileCoord::new(counter, 0, 0),
                TileData::Terrain(TerrainData {
                    heights: vec![0.0; 33 * 33],
                    width: 33,
                    height: 33,
                }),
            );
            counter += 1;
        });
    });

    group.finish();
}

fn bench_terrain_mesh_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("terrain_mesh_gen");

    for resolution in [8, 16, 32, 64] {
        let verts = (resolution + 1) * (resolution + 1);
        let heights: Vec<f32> = (0..verts)
            .map(|i| (i as f32 * 0.1).sin() * 100.0)
            .collect();
        let coord = TileCoord::new(0, 0, 10);

        group.bench_with_input(
            BenchmarkId::from_parameter(resolution),
            &resolution,
            |b, &res| {
                b.iter(|| {
                    black_box(generate_tile_mesh(coord, black_box(&heights), res, 1.0, 1.0));
                });
            },
        );
    }

    group.finish();
}

fn bench_terrain_rgb_decode(c: &mut Criterion) {
    // Simulate decoding a 256x256 terrain tile
    let tile_size = 256 * 256;
    let rgb_data: Vec<(u8, u8, u8)> = (0..tile_size)
        .map(|i| {
            let v = (i as u32 * 37) % 16777216; // pseudo-random
            ((v >> 16) as u8, ((v >> 8) & 0xFF) as u8, (v & 0xFF) as u8)
        })
        .collect();

    c.bench_function("terrain_rgb_decode_256x256", |b| {
        b.iter(|| {
            let mut heights = Vec::with_capacity(tile_size);
            for &(r, g, bb) in &rgb_data {
                heights.push(decode_terrain_rgb(r, g, bb));
            }
            black_box(heights);
        });
    });
}

criterion_group!(
    benches,
    bench_tile_cache,
    bench_terrain_mesh_generation,
    bench_terrain_rgb_decode
);
criterion_main!(benches);
