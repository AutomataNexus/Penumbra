<p align="center">
  <img src="assets/logo.png" alt="Penumbra" width="280">
</p>

<h1 align="center">Penumbra</h1>

<p align="center"><strong>General-purpose 3D rendering SDK for Rust.</strong></p>

<p align="center">
  <a href="https://github.com/AutomataNexus/Penumbra/actions/workflows/ci.yml"><img src="https://github.com/AutomataNexus/Penumbra/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.85+-orange?logo=rust" alt="Rust 1.85+">
  <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue" alt="License">
  <img src="https://img.shields.io/badge/crates-18-green" alt="18 Crates">
  <img src="https://img.shields.io/badge/tests-116-brightgreen" alt="116 Tests">
  <img src="https://img.shields.io/badge/WASM-ready-blueviolet" alt="WASM Ready">
  <img src="https://img.shields.io/badge/backend-wgpu%2024-informational" alt="wgpu 24">
</p>

Penumbra sits between raw GPU bindings (wgpu) and full game engines (Bevy). It provides a complete 3D rendering toolkit that works natively on desktop (Vulkan, Metal, DX12) and in the browser (WebGPU, WebGL2 via WASM), with no game engine baggage, no ECS mandate, and no opinionated application framework.

---

## Architecture

```
+------------------------------------------------------------------+
|                        Your Application                           |
|                                                                   |
|  // Scene graph mode             // Immediate mode                |
|  let node = scene.add_mesh(...); renderer.draw_line(a, b, color); |
|  scene.render(&camera, &mut r);  renderer.draw_billboard(...);    |
+-------------------------------+----------------------------------+
                                |
          +---------------------v-----------------------+
          |              penumbra-scene                  |
          |  Scene graph, node tree, transform hierarchy |
          |  Frustum culling, LOD selection              |
          +---------------------+-----------------------+
                                |
          +---------------------v-----------------------+
          |              penumbra-core                   |
          |  Renderer, RenderFrame, DrawCall             |
          |  Mesh, Material, Texture, Buffer abstractions|
          +---------------------+-----------------------+
                                |
          +---------------------v-----------------------+
          |         Render Feature Crates                |
          |                                              |
          |  penumbra-pbr       penumbra-instance        |
          |  penumbra-terrain   penumbra-atmosphere      |
          |  penumbra-post      penumbra-shadow          |
          |  penumbra-text      penumbra-compute         |
          |  penumbra-immediate penumbra-camera          |
          +---------------------+-----------------------+
                                |
          +---------------------v-----------------------+
          |            penumbra-backend                  |
          |          RenderBackend trait                  |
          +----------+---------------------+------------+
                     |                     |
            +--------v-------+    +--------v-----------+
            |  penumbra-wgpu |    | (your custom       |
            |  Vulkan/Metal/ |    |  backend)           |
            |  DX12/WebGPU/  |    |  implements         |
            |  WebGL2        |    |  RenderBackend      |
            +----------------+    +--------------------+
```

## Render Pipeline

```
Geometry pass (G-buffer: albedo, normal, metallic-roughness, emissive, depth)
  |
  v
SSAO pass (optional, reads depth + normals)
  |
  v
Shadow map pass (directional: cascaded, point: cube map)
  |
  v
Lighting pass (PBR BRDF + IBL + punctual lights)
  |
  v
Transparent pass (forward, sorted back-to-front)
  |
  v
Atmosphere / sky pass
  |
  v
Post-processing chain (tone mapping, bloom, FXAA/TAA, color grading)
  |
  v
2D overlay pass (immediate mode 2D, HUD, text)
  |
  v
Present
```

## Features

- **Backend agnostic** -- RenderBackend trait abstracts over GPU APIs; default wgpu implementation
- **Scene graph + immediate mode** -- both composable in the same frame
- **PBR rendering** -- Cook-Torrance BRDF, IBL, directional/point/spot lights
- **27K+ entity instancing** -- GPU frustum culling, single draw call per entity type
- **Tile-based terrain** -- satellite imagery streaming (XYZ, Mapbox), elevation displacement
- **Atmospheric sky** -- Bruneton-Neyret scattering, sun/moon/stars, fog
- **Cascaded shadow maps** -- PCF soft shadows, point light cube maps
- **Post-processing** -- tone mapping (ACES), bloom, SSAO, FXAA, TAA, color grading
- **SDF text rendering** -- crisp at all scales, batched labels for 27K entities
- **Compute shaders** -- GPU culling, tile decompression, custom workloads
- **Geospatial** -- WGS84, double-precision, ECEF/ENU conversions, tile math
- **WASM / browser** -- WebGPU + WebGL2 fallback
- **Camera system** -- perspective, orthographic, orbit, fly, globe controllers

## Crate Overview

| Crate | Description |
|-------|-------------|
| `penumbra-core` | Renderer, RenderFrame, core types, math re-exports |
| `penumbra-backend` | RenderBackend trait (GPU abstraction) |
| `penumbra-wgpu` | Default wgpu backend (Vulkan/Metal/DX12/WebGPU/WebGL2) |
| `penumbra-scene` | Scene graph, transform hierarchy, frustum culling, LOD |
| `penumbra-pbr` | PBR pipeline, lights, IBL, Cook-Torrance BRDF |
| `penumbra-instance` | GPU-accelerated instanced rendering (27K+ entities) |
| `penumbra-terrain` | Tile-based satellite imagery and terrain streaming |
| `penumbra-atmosphere` | Bruneton-Neyret atmospheric scattering, fog |
| `penumbra-post` | Post-processing pipeline (tone mapping, bloom, SSAO, FXAA) |
| `penumbra-shadow` | Cascaded shadow maps, PCF, point light shadows |
| `penumbra-text` | SDF font rendering, billboard text, batched labels |
| `penumbra-compute` | Compute shader abstraction |
| `penumbra-geo` | WGS84 geodesy, coordinate conversions, tile math |
| `penumbra-immediate` | Immediate mode draw API (lines, shapes, billboards) |
| `penumbra-camera` | Camera types, orbit/fly/globe controllers, raycasting |
| `penumbra-asset` | Asset loading (glTF 2.0, OBJ, PNG/JPEG, HDR, TTF) |
| `penumbra-winit` | winit window integration |
| `penumbra-web` | WASM / browser target support |

## Quick Start

```rust
use penumbra_core::{Renderer, RendererConfig};
use penumbra_wgpu::WgpuBackend;
use penumbra_camera::PerspectiveCamera;
use penumbra_scene::Scene;

// Create backend and renderer
let backend = WgpuBackend::headless(1280, 720, Default::default()).unwrap();
let mut renderer = Renderer::new(backend, RendererConfig::default());

// Set up scene
let mut scene = Scene::new();
let mesh_id = renderer.create_mesh(cube_mesh()).unwrap();
let mat_id = renderer.add_material(Default::default());
scene.add_mesh(mesh_id.id, mat_id);

// Render loop
let mut frame = renderer.begin_frame();
// ... submit draw calls ...
renderer.end_frame(frame);
```

## Build

```bash
# Check all crates
cargo check

# Run tests
cargo test

# Clippy
cargo clippy -- -D warnings

# Build specific crate
cargo build -p penumbra-core
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---

*Andrew Jewell Sr. -- AutomataNexus LLC -- devops@automatanexus.com*
