# Penumbra -- Performance Targets & Benchmarks

## Hardware

Benchmarked on:
- **GPU:** NVIDIA GeForce RTX 5070 Ti Laptop (12GB VRAM), Vulkan backend
- **CPU:** Intel (via Windows native, release mode)
- **OS:** Windows 11 (native, not WSL2)

## GPU Rendering Results

Real GPU frame times measured through the full Vulkan render pipeline at 1920x1080 with depth testing. Each entity is an instanced cube (12 triangles, 36 vertices) rendered in a single draw call.

| Scenario | Target | Median | FPS | P99 | Result |
|----------|--------|--------|-----|-----|--------|
| 27K instanced entities | < 8ms | **0.132 ms** | **7,593** | 0.339 ms | PASS |
| 10K instanced entities | < 8ms | 0.084 ms | 11,848 | 0.188 ms | PASS |
| 1K instanced entities | < 8ms | 0.068 ms | 14,599 | 0.281 ms | PASS |
| 50K instanced entities (stress) | < 8ms | 0.161 ms | 6,207 | 0.434 ms | PASS |
| Empty frame (baseline) | -- | 0.066 ms | 15,106 | 0.125 ms | -- |

**27K entities at 1080p: 0.132ms median frame time. 60x faster than the 8ms target.**

The 8ms GPU budget at 60fps provides room for the full rendering stack on top of instanced geometry:
- PBR lighting pass: estimated 1-2ms (deferred, single pass regardless of light count)
- Shadow map pass (4 cascades): estimated 1-2ms
- Post-processing chain: estimated 0.5-1ms
- **Total estimated: 3-5ms, well within the 8ms budget**

## CPU-Side Results

These measure the CPU-side frame preparation overhead -- the work done before GPU commands are issued.

### Scene Graph (transform propagation)

| Nodes | Time | Per-node |
|-------|------|----------|
| 100 | 622 ns | 6.2 ns |
| 1,000 | 6.1 us | 6.1 ns |
| 5,000 | 31 us | 6.2 ns |

### Instance Management

| Operation | 1K | 10K | 27K |
|-----------|-----|------|------|
| Generate InstanceData | 12.9 us | 128 us | 708 us |
| Batch update | 1.4 us | 47 us | 772 us |
| CPU frustum cull | 1.5 us | 12.9 us | 35 us |

### Full Frame Preparation (27K entities)

| Operation | Time |
|-----------|------|
| CPU frustum cull + light uniforms + text layout | 38.5 us |

At 38.5 us CPU + 0.132 ms GPU per frame, total frame time is **0.17ms**.
This is **1.0% of the 16.6ms budget at 60fps**.

### Tile Streaming

| Operation | Time |
|-----------|------|
| Tile cache insert (256 tiles) | 34 us |
| Tile cache lookup (hit) | 6.2 ns |
| LRU eviction under pressure | 90 ns |
| Terrain mesh gen (8x8) | 643 ns |
| Terrain mesh gen (16x16) | 1.8 us |
| Terrain mesh gen (32x32) | 6.7 us |
| Terrain mesh gen (64x64) | 26 us |
| Terrain-RGB decode (256x256) | 50 us |

## Benchmark Methodology

CPU benchmarks use criterion. GPU benchmarks use `device.poll(Maintain::Wait)` to measure wall-clock time including GPU completion. All benchmarks run in release mode.

```bash
cargo bench                                  # CPU benchmarks (criterion)
cargo run --bin gpu_render_bench --release    # GPU benchmarks (Vulkan)
cargo run --bin gpu_probe --release           # GPU adapter probe
```

## GPU Adapter Info

```
NVIDIA GeForce RTX 5070 Ti Laptop GPU (Vulkan, DiscreteGpu)
  max_texture_size: 32768
  max_buffer_size: 18446744073709551615
  compute: true
  timestamp_query: true
```
