# Penumbra -- Performance Targets & Benchmarks

## Hard Requirements (GPU)

| Scenario | Target | Hardware | Measured | Status |
|----------|--------|----------|----------|--------|
| 27K instanced entities, frustum culled | 60fps | GTX 1060 / M1 | -- | pending GPU |
| Full globe view, zoom 12, satellite tiles | 60fps | GTX 1060 / M1 | -- | pending GPU |
| Atmosphere + terrain + 27K entities + post | 60fps | GTX 1060 / M1 | -- | pending GPU |
| Same scene in browser (WebGPU) | 60fps | Chrome on M1 | -- | pending GPU |
| Same scene in browser (WebGL2 fallback) | 30fps | Chrome on Intel UHD | -- | pending GPU |
| Initial tile load (cold cache, zoom 8) | < 3s | -- | -- | pending GPU |
| Frame time (GPU), 27K entities | < 8ms | GTX 1060 | -- | pending GPU |
| Memory (GPU), 27K entities + terrain | < 1.5GB | -- | -- | pending GPU |

## How 60fps at 27K Entities Is Achieved

1. GPU-side frustum culling via compute -- visible entities only reach the draw stage
2. Single instanced draw call per entity type (typically 2-4 draw calls for all entities)
3. Instance buffer updated with `write_buffer` -- single CPU to GPU transfer per frame
4. SDF text labels batched -- single draw call for all visible labels
5. Tile cache prevents redundant texture uploads -- tiles stay on GPU until evicted

## Benchmark Methodology

Benchmarks use criterion. Run with:

```bash
cargo bench --bench instancing      # 27K entity throughput
cargo bench --bench tile_streaming  # tile streaming bandwidth
cargo bench --bench full_scene      # full scene composite
cargo bench                         # all benchmarks
```

Regression > 5% from documented target fails CI.

## CPU-Side Benchmark Results

Measured on Linux x86_64 (CI runner, no GPU). These measure the CPU-side
frame preparation overhead -- the work done before any GPU commands are issued.

### Scene Graph (transform propagation)

| Nodes | Time | Per-node |
|-------|------|----------|
| 100 | 651 ns | 6.5 ns |
| 1,000 | 6.6 us | 6.6 ns |
| 5,000 | 34 us | 6.8 ns |

### Instance Management

| Operation | 1K | 10K | 27K |
|-----------|-----|------|------|
| Generate InstanceData | 8.2 us | 79 us | 218 us |
| Batch update (Vec copy) | 1.3 us | 17 us | 96 us |
| CPU frustum cull | 1.6 us | 15 us | 42 us |

### Full Frame Preparation (27K entities)

| Operation | Time |
|-----------|------|
| CPU frustum cull + light uniforms + text layout | 41 us |

At 41 us CPU overhead per frame, the CPU budget at 60fps (16.6ms) is 0.25% utilized.
This leaves 16.5ms of GPU headroom per frame.

### Tile Streaming

| Operation | Time |
|-----------|------|
| Tile cache insert (256 tiles) | 29 us |
| Tile cache lookup (hit) | 6.7 ns |
| LRU eviction under pressure | 104 ns |
| Terrain mesh gen (8x8) | 634 ns |
| Terrain mesh gen (16x16) | 1.4 us |
| Terrain mesh gen (32x32) | 5.0 us |
| Terrain mesh gen (64x64) | 18 us |
| Terrain-RGB decode (256x256) | 78 us |

## GPU Benchmark Results

Pending -- requires GPU-equipped runner. Will be populated when benchmarks
run on hardware with Vulkan/Metal/DX12 support.
