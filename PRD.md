# Penumbra — General Purpose 3D Rendering SDK for Rust
## Product Requirements Document v1.0
**Owner:** Andrew Jewell Sr. — AutomataNexus LLC
**Contact:** devops@automatanexus.com
**Classification:** Open Source — crates.io (HOLD — private until release authorized)
**License:** MIT OR Apache-2.0
**Status:** Pre-Development
**Last Updated:** 2026-03-30

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Background & Motivation](#2-background--motivation)
3. [How Penumbra Fills the Gap](#3-how-penumbra-fills-the-gap)
4. [Architecture Overview](#4-architecture-overview)
5. [Crate Structure](#5-crate-structure)
   - 5.1 [penumbra-core](#51-penumbra-core)
   - 5.2 [penumbra-backend](#52-penumbra-backend)
   - 5.3 [penumbra-wgpu](#53-penumbra-wgpu)
   - 5.4 [penumbra-scene](#54-penumbra-scene)
   - 5.5 [penumbra-pbr](#55-penumbra-pbr)
   - 5.6 [penumbra-instance](#56-penumbra-instance)
   - 5.7 [penumbra-terrain](#57-penumbra-terrain)
   - 5.8 [penumbra-atmosphere](#58-penumbra-atmosphere)
   - 5.9 [penumbra-post](#59-penumbra-post)
   - 5.10 [penumbra-shadow](#510-penumbra-shadow)
   - 5.11 [penumbra-text](#511-penumbra-text)
   - 5.12 [penumbra-compute](#512-penumbra-compute)
   - 5.13 [penumbra-geo](#513-penumbra-geo)
   - 5.14 [penumbra-immediate](#514-penumbra-immediate)
   - 5.15 [penumbra-camera](#515-penumbra-camera)
   - 5.16 [penumbra-asset](#516-penumbra-asset)
   - 5.17 [penumbra-winit](#517-penumbra-winit)
   - 5.18 [penumbra-web](#518-penumbra-web)
6. [Render Backend System](#6-render-backend-system)
7. [Scene Graph](#7-scene-graph)
8. [Immediate Mode API](#8-immediate-mode-api)
9. [PBR Pipeline](#9-pbr-pipeline)
10. [Instanced Rendering](#10-instanced-rendering)
11. [Tile-Based Streaming](#11-tile-based-streaming)
12. [Atmosphere & Sky](#12-atmosphere--sky)
13. [Post-Processing Pipeline](#13-post-processing-pipeline)
14. [Shadow System](#14-shadow-system)
15. [Text Rendering](#15-text-rendering)
16. [2D + 3D Compositing](#16-2d--3d-compositing)
17. [Compute Shaders](#17-compute-shaders)
18. [Camera System](#18-camera-system)
19. [Asset Pipeline](#19-asset-pipeline)
20. [WASM / Browser Target](#20-wasm--browser-target)
21. [NexusPulse Tactical Integration](#21-nexuspulse-tactical-integration)
22. [Performance Targets](#22-performance-targets)
23. [Tech Stack](#23-tech-stack)
24. [Directory Structure](#24-directory-structure)
25. [Development Phases](#25-development-phases)
26. [Test Strategy](#26-test-strategy)
27. [Acceptance Criteria](#27-acceptance-criteria)
28. [Claude Code Operational Instructions](#28-claude-code-operational-instructions)

---

## 1. Executive Summary

Penumbra is a **general-purpose 3D rendering SDK for Rust** — not a game engine, not raw GPU bindings, but the layer in between. It sits where Three.js and Babylon.js sit in the JavaScript ecosystem: a complete, production-ready 3D rendering toolkit that works natively on desktop (Vulkan, Metal, DirectX 12) and in the browser (WebGPU, WebGL2 via WASM), with no game engine baggage, no ECS mandate, and no opinionated application framework.

Penumbra is render-backend-agnostic by design. The `RenderBackend` trait abstracts over GPU APIs — the default implementation is wgpu (covering all platforms), but any backend can be plugged in. Applications that already use a specific GPU API can write a thin adapter and use Penumbra's entire rendering feature set on top of their existing pipeline.

Two scene management models are supported simultaneously and composably:

1. **Scene graph** — hierarchical node tree with transforms, parent/child relationships, and retained-mode rendering. Correct for complex 3D scenes with many objects.

2. **Immediate mode** — frame-by-frame draw call API. Correct for dynamic content, debug overlays, HUD elements, and simple use cases where managing a scene graph is overkill.

Both modes can be used in the same frame, in the same application.

The feature set covers the full stack needed to build a 3D geospatial application from scratch: PBR rendering, 27K+ entity instanced rendering, tile-based satellite imagery and terrain streaming, atmospheric sky rendering, cascaded shadow maps, post-processing pipeline, text rendering, 2D/3D compositing, and compute shaders — all running at 60fps on target hardware.

Penumbra is the rendering foundation for NexusPulse Tactical's 3D globe view, but is designed as a fully general SDK usable by any Rust application.

---

## 2. Background & Motivation

### 2.1 The Missing Layer

The Rust 3D rendering ecosystem has two layers:

**Layer 1 — Raw GPU bindings (too low):**
wgpu, ash (Vulkan), metal-rs, d3d12. These give you direct GPU access with Rust safety guarantees. But you're writing WGSL shaders, managing buffer lifetimes, building render pipelines from scratch. Building a 3D application on raw wgpu is like building a web app in raw TCP sockets.

**Layer 2 — Full game engines (too high):**
Bevy, Fyrox, Macroquad. These are complete application frameworks with ECS world models, asset servers, plugin systems, input handling, audio, UI — the works. Adopting Bevy means adopting its entire worldview. If you're building a geospatial SDK, a data visualization tool, or a custom 3D application, you're fighting the game engine the whole time.

**The missing layer — a 3D rendering SDK:**
Three.js has dominated JavaScript 3D for 15 years because it sits precisely in this gap: above raw WebGL, below a full game engine. You get a scene graph, a camera, materials, lights, loaders — and nothing else. Your application structure is your own.

Penumbra is this layer for Rust.

### 2.2 Why Now

- wgpu has reached production maturity — Firefox, Servo, and Deno ship it as their WebGPU implementation
- WASM compilation of Rust is production-ready — wgpu runs in browsers via WebGPU/WebGL2
- The geospatial/defense/simulation market needs a Rust-native 3D SDK — CesiumJS is JavaScript, cesium-rs doesn't exist at production quality
- NexusPulse Tactical's 3D globe requirement is the immediate driver — no existing Rust solution handles satellite tiles + terrain + 27K entities at 60fps

---

## 3. How Penumbra Fills the Gap

| | wgpu (raw) | Bevy (engine) | Three.js (JS) | Penumbra |
|--|-----------|--------------|--------------|---------|
| Language | Rust | Rust | JavaScript | Rust |
| Level | GPU bindings | Full engine | SDK | SDK |
| Scene graph | No | ECS | Yes | Yes |
| Immediate mode | No | No | Partial | Yes |
| PBR | DIY | Yes | Yes | Yes |
| Instanced 27K+ | DIY | Partial | Yes | Yes |
| Tile streaming | No | No | Via plugin | Yes |
| Atmosphere | No | Plugin | Plugin | Yes |
| Post-processing | DIY | Plugin | Yes | Yes |
| Shadows | DIY | Yes | Yes | Yes |
| Text | No | Plugin | Via drei | Yes |
| 2D + 3D | DIY | Partial | Yes | Yes |
| Compute shaders | Yes | Partial | No | Yes |
| WASM | Yes | Yes | N/A | Yes |
| Pluggable backend | N/A | No | No | Yes |
| Geospatial (WGS84) | No | No | No | Yes |
| No engine baggage | N/A | No | N/A | Yes |
| crates.io | Yes | Yes | N/A | Yes |

---

## 4. Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                      Your Application                             │
│                                                                   │
│  // Scene graph mode           // Immediate mode                  │
│  let node = scene.add_mesh(…); renderer.draw_line(a, b, color);  │
│  scene.render(&mut renderer);  renderer.draw_billboard(…);       │
└──────────────────────┬───────────────────────────────────────────┘
                       │
    ┌──────────────────▼──────────────────────────────┐
    │               penumbra-scene                     │
    │  Scene graph, node tree, transform hierarchy     │
    │  Frustum culling, LOD selection, draw call sort  │
    └──────────────────┬──────────────────────────────┘
                       │
    ┌──────────────────▼──────────────────────────────┐
    │              penumbra-core                       │
    │  RenderBackend trait, RenderFrame, DrawCall      │
    │  Mesh, Material, Texture, Buffer abstractions    │
    └──────────────────┬──────────────────────────────┘
                       │
    ┌──────────────────▼──────────────────────────────┐
    │  Render Feature Crates                           │
    │                                                  │
    │  penumbra-pbr        penumbra-instance           │
    │  penumbra-terrain    penumbra-atmosphere         │
    │  penumbra-post       penumbra-shadow             │
    │  penumbra-text       penumbra-compute            │
    │  penumbra-immediate  penumbra-camera             │
    └──────────────────┬──────────────────────────────┘
                       │
    ┌──────────────────▼──────────────────────────────┐
    │              penumbra-backend                    │
    │           RenderBackend trait                    │
    └──────────────────┬──────────────────────────────┘
                       │
         ┌─────────────┴─────────────┐
         ▼                           ▼
  penumbra-wgpu               (your custom backend)
  (default implementation)    implements RenderBackend
  Vulkan/Metal/DX12/
  WebGPU/WebGL2
```

### 4.1 Core Design Principles

**Backend agnosticism is a first-class constraint** — every rendering feature crate depends only on `penumbra-backend`'s `RenderBackend` trait, never on wgpu directly. Swapping backends requires zero changes to feature crates.

**No application framework** — Penumbra has no event loop, no window management, no input system, no audio, no ECS. It renders. Everything else is your problem. Thin integration crates (`penumbra-winit`, `penumbra-web`) provide optional bridges to common windowing libraries.

**Scene graph and immediate mode are peers** — neither is built on top of the other. The scene graph produces draw calls. Immediate mode produces draw calls. Both are submitted to the same `RenderFrame` in the same frame. Compositing is free.

**Geospatial is a first-class concern** — WGS84 coordinate system, double-precision positions, tile-based streaming, and terrain are built into the SDK, not bolted on as an afterthought. This is the differentiator that no existing Rust 3D library provides.

**60fps at 27K+ entities is a hard requirement** — instanced rendering, GPU frustum culling, LOD, and tile streaming are designed from day one around this constraint.

---

## 5. Crate Structure

### 5.1 penumbra-core

Foundation types and the `RenderBackend` trait. Zero dependency on any specific GPU API.

**Key types:**

```rust
/// The central renderer — owns the backend, manages frame lifecycle
pub struct Renderer {
    backend:     Box<dyn RenderBackend>,
    config:      RendererConfig,
    frame_stats: FrameStats,
}

impl Renderer {
    pub fn new(backend: impl RenderBackend + 'static, config: RendererConfig) -> Self;
    pub fn begin_frame(&mut self) -> RenderFrame;
    pub fn end_frame(&mut self, frame: RenderFrame);
    pub fn resize(&mut self, width: u32, height: u32);
    pub fn stats(&self) -> &FrameStats;
}

/// A single frame being rendered — all draw calls go through this
pub struct RenderFrame {
    pub width:   u32,
    pub height:  u32,
    pub time:    f64,       // seconds since renderer creation
    pub delta:   f32,       // seconds since last frame
    pub camera:  CameraUniforms,
    draws:       Vec<DrawCall>,
}

pub struct RendererConfig {
    pub width:            u32,
    pub height:           u32,
    pub msaa_samples:     u32,       // 1, 2, 4, 8
    pub hdr:              bool,
    pub vsync:            bool,
    pub max_instances:    u32,       // default 65536
    pub tile_cache_mb:    u32,       // tile texture cache size
}

/// GPU-side mesh — vertices + indices uploaded to GPU
pub struct GpuMesh {
    pub id:          MeshId,
    pub vertex_count: u32,
    pub index_count:  u32,
    pub aabb:        Aabb,
}

/// GPU-side texture
pub struct GpuTexture {
    pub id:     TextureId,
    pub width:  u32,
    pub height: u32,
    pub format: TextureFormat,
}

/// A material — PBR parameters + texture references
pub struct Material {
    pub id:                MaterialId,
    pub albedo:            Rgba,
    pub albedo_texture:    Option<TextureId>,
    pub metallic:          f32,
    pub roughness:         f32,
    pub metallic_roughness_texture: Option<TextureId>,
    pub normal_texture:    Option<TextureId>,
    pub emissive:          Rgb,
    pub emissive_texture:  Option<TextureId>,
    pub occlusion_texture: Option<TextureId>,
    pub alpha_mode:        AlphaMode,
    pub double_sided:      bool,
}

/// AABB for culling
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

/// Frame performance stats
pub struct FrameStats {
    pub frame_time_ms:    f32,
    pub fps:              f32,
    pub draw_calls:       u32,
    pub triangles:        u64,
    pub instances:        u32,
    pub tiles_loaded:     u32,
    pub tiles_streaming:  u32,
    pub gpu_memory_mb:    u32,
}
```

**Math types** (re-exported from `glam`):
`Vec2`, `Vec3`, `Vec4`, `Mat3`, `Mat4`, `Quat`, `DVec3` (double precision for geospatial positions)

---

### 5.2 penumbra-backend

The `RenderBackend` trait — the only interface between Penumbra's feature crates and the GPU.

```rust
pub trait RenderBackend: Send + Sync {
    // Resource creation
    fn create_mesh(&mut self, desc: MeshDescriptor)       -> Result<GpuMesh, BackendError>;
    fn create_texture(&mut self, desc: TextureDescriptor) -> Result<GpuTexture, BackendError>;
    fn create_buffer(&mut self, desc: BufferDescriptor)   -> Result<GpuBuffer, BackendError>;
    fn create_pipeline(&mut self, desc: PipelineDescriptor) -> Result<PipelineId, BackendError>;
    fn create_compute_pipeline(&mut self, desc: ComputePipelineDescriptor)
        -> Result<ComputePipelineId, BackendError>;

    // Resource destruction
    fn destroy_mesh(&mut self, id: MeshId);
    fn destroy_texture(&mut self, id: TextureId);
    fn destroy_buffer(&mut self, id: BufferId);

    // Buffer updates
    fn write_buffer(&mut self, id: BufferId, offset: u64, data: &[u8]);
    fn read_buffer(&mut self, id: BufferId, offset: u64, len: u64) -> Vec<u8>;

    // Texture updates
    fn write_texture(&mut self, id: TextureId, region: TextureRegion, data: &[u8]);

    // Frame lifecycle
    fn begin_frame(&mut self) -> Result<(), BackendError>;
    fn end_frame(&mut self)   -> Result<(), BackendError>;
    fn present(&mut self)     -> Result<(), BackendError>;

    // Render pass
    fn begin_render_pass(&mut self, desc: RenderPassDescriptor) -> RenderPassHandle;
    fn end_render_pass(&mut self, handle: RenderPassHandle);
    fn set_pipeline(&mut self, handle: RenderPassHandle, pipeline: PipelineId);
    fn set_bind_group(&mut self, handle: RenderPassHandle, index: u32, group: BindGroupId);
    fn set_vertex_buffer(&mut self, handle: RenderPassHandle, slot: u32, buffer: BufferSlice);
    fn set_index_buffer(&mut self, handle: RenderPassHandle, buffer: BufferSlice);
    fn draw(&mut self, handle: RenderPassHandle, vertices: Range<u32>, instances: Range<u32>);
    fn draw_indexed(&mut self, handle: RenderPassHandle, indices: Range<u32>,
                    base_vertex: i32, instances: Range<u32>);

    // Compute pass
    fn begin_compute_pass(&mut self) -> ComputePassHandle;
    fn end_compute_pass(&mut self, handle: ComputePassHandle);
    fn set_compute_pipeline(&mut self, handle: ComputePassHandle, pipeline: ComputePipelineId);
    fn dispatch(&mut self, handle: ComputePassHandle, x: u32, y: u32, z: u32);

    // Capabilities
    fn capabilities(&self) -> BackendCapabilities;
    fn backend_name(&self) -> &str;
}

pub struct BackendCapabilities {
    pub max_texture_size:      u32,
    pub max_buffer_size:       u64,
    pub max_instances:         u32,
    pub supports_compute:      bool,
    pub supports_indirect:     bool,
    pub supports_timestamp_queries: bool,
    pub supports_hdr:          bool,
}
```

---

### 5.3 penumbra-wgpu

The default backend implementation. Implements `RenderBackend` using wgpu.

Targets:
- **Native:** Vulkan (Linux, Windows, Android), Metal (macOS, iOS), DirectX 12 (Windows)
- **Browser:** WebGPU (Chrome, Firefox via flag), WebGL2 (universal fallback)

WGSL shaders shared across all targets. Naga handles cross-compilation to SPIR-V / MSL / HLSL as needed.

**Surface creation:**

```rust
// From a winit window
let backend = WgpuBackend::from_window(&window, WgpuConfig::default()).await?;

// From a raw wgpu surface (custom windowing)
let backend = WgpuBackend::from_surface(surface, adapter, device, queue, WgpuConfig::default());

// Headless (for testing, offscreen rendering)
let backend = WgpuBackend::headless(width, height, WgpuConfig::default()).await?;
```

**wgpu feature gate parity:**

All Penumbra features work with both WebGPU-capable hardware and WebGL2 fallback. Features that require compute shaders (GPU culling, tile decompression) gracefully degrade to CPU fallback on WebGL2.

---

### 5.4 penumbra-scene

The scene graph. Hierarchical node tree with transforms, retained-mode rendering, frustum culling, and LOD.

**Scene graph structure:**

```rust
pub struct Scene {
    nodes:    SlotMap<NodeId, SceneNode>,
    root:     NodeId,
    dirty:    DirtyFlags,
}

pub struct SceneNode {
    pub transform:  Transform,
    pub world_transform: Mat4,    // computed, cached
    pub parent:     Option<NodeId>,
    pub children:   Vec<NodeId>,
    pub renderable: Option<Renderable>,
    pub visible:    bool,
    pub name:       Option<String>,
    pub user_data:  Option<Box<dyn Any + Send + Sync>>,
}

pub struct Transform {
    pub translation: Vec3,
    pub rotation:    Quat,
    pub scale:       Vec3,
}

/// What this node renders
pub enum Renderable {
    Mesh     { mesh: MeshId, material: MaterialId },
    Instance { batch: InstanceBatchId },
    Terrain  { tile_source: TileSourceId },
    Text     { text: TextRendererId },
    Light    { light: Light },
    Custom   { renderer: Box<dyn CustomRenderer> },
}
```

**Scene API:**

```rust
let mut scene = Scene::new();

// Add a mesh node
let node = scene.add_mesh(mesh_id, material_id);
scene.set_transform(node, Transform {
    translation: Vec3::new(1.0, 0.0, 0.0),
    rotation:    Quat::IDENTITY,
    scale:       Vec3::ONE,
});

// Parent/child hierarchy
let parent = scene.add_empty();
let child  = scene.add_mesh(mesh_id, material_id);
scene.set_parent(child, parent);

// Lights
let sun = scene.add_light(Light::Directional {
    direction: Vec3::new(-0.5, -1.0, -0.3).normalize(),
    color:     Rgb::new(1.0, 0.98, 0.95),
    intensity: 10.0,
    cast_shadows: true,
});

// Render the scene
scene.render(&camera, &mut renderer);
```

**Transform propagation:**

World transforms are computed lazily — only dirty nodes and their descendants are recomputed per frame. For scenes with many static nodes (terrain, buildings), this avoids redundant matrix multiplications.

**Frustum culling:**

Every `Renderable::Mesh` node has an AABB. The scene graph performs frustum culling against the camera's view frustum before generating draw calls. Culled nodes produce zero draw calls.

**LOD system:**

```rust
pub struct LodMesh {
    pub levels: Vec<LodLevel>,
}

pub struct LodLevel {
    pub mesh:           MeshId,
    pub max_screen_size: f32,   // switch to this level when screen-space size < threshold
}

scene.add_lod_mesh(lod_mesh, material_id);
```

---

### 5.5 penumbra-pbr

Physically Based Rendering pipeline. Cook-Torrance BRDF, image-based lighting (IBL), punctual lights.

**Light types:**

```rust
pub enum Light {
    Directional {
        direction:    Vec3,
        color:        Rgb,
        intensity:    f32,   // lux
        cast_shadows: bool,
    },
    Point {
        position:     Vec3,
        color:        Rgb,
        intensity:    f32,   // candela
        range:        f32,
        cast_shadows: bool,
    },
    Spot {
        position:     Vec3,
        direction:    Vec3,
        color:        Rgb,
        intensity:    f32,
        range:        f32,
        inner_angle:  f32,
        outer_angle:  f32,
        cast_shadows: bool,
    },
}
```

**IBL (Image-Based Lighting):**

```rust
// Load an HDR environment for IBL
let sky = renderer.load_hdr_sky("sky.hdr")?;
renderer.set_environment(sky, EnvironmentConfig {
    intensity:    1.0,
    rotation:     0.0,
    diffuse_only: false,
});
```

**Material system:**

Full glTF 2.0 PBR material model:
- Metallic-roughness workflow
- Albedo, normal, metallic-roughness, occlusion, emissive texture maps
- Alpha mode: Opaque, Mask (with cutoff), Blend
- Double-sided rendering
- KHR extensions: clearcoat, transmission, volume, sheen, iridescence

**WGSL shaders:**

PBR shaders written in WGSL, compiled once, run on all backends. Shader variants generated at compile time via Rust const generics — no runtime shader compilation.

---

### 5.6 penumbra-instance

GPU-accelerated instanced rendering. 27K+ entities at 60fps.

**Instance batch:**

```rust
pub struct InstanceBatch {
    pub id:       InstanceBatchId,
    pub mesh:     MeshId,
    pub material: MaterialId,
    pub capacity: u32,          // pre-allocated GPU buffer size
}

// Create a batch
let batch = renderer.create_instance_batch(InstanceBatchDesc {
    mesh:     unit_cone_mesh,
    material: track_material,
    capacity: 32768,            // reserve for 32K instances
})?;

// Update instances — uploaded to GPU in one buffer write
renderer.update_instances(batch, &instances)?;
// instances: &[InstanceData]

pub struct InstanceData {
    pub transform:  Mat4,
    pub color:      Rgba,
    pub uv_offset:  Vec2,    // atlas UV offset for icon batches
    pub uv_scale:   Vec2,    // atlas UV scale
    pub user_data:  [f32; 4], // custom per-instance data (passed to shader)
}
```

**GPU frustum culling:**

On backends that support compute (WebGPU, Vulkan, Metal, DX12), frustum culling runs entirely on the GPU:

1. Upload all instance transforms to a GPU buffer
2. Dispatch compute shader: for each instance, test AABB against camera frustum
3. Compute shader writes visible instance indices to an indirect draw buffer
4. Draw call uses `draw_indirect` — zero CPU readback

On WebGL2 (no compute), CPU frustum culling is used as fallback.

**Performance at 27K entities:**

27,000 instances × 64 bytes per `InstanceData` = 1.7MB buffer write per frame.
At 60fps = 102MB/s GPU upload bandwidth — well within wgpu limits on all platforms.
With GPU frustum culling: typically 1–3 draw calls per batch regardless of entity count.

---

### 5.7 penumbra-terrain

Tile-based streaming for satellite imagery and terrain elevation. Supports XYZ, TMS, WMTS, and custom tile sources.

**Tile source abstraction:**

```rust
#[async_trait]
pub trait TileSource: Send + Sync {
    async fn fetch_tile(&self, coord: TileCoord) -> Result<TileData, TileError>;
    fn tile_format(&self) -> TileFormat;
    fn attribution(&self) -> Option<&str>;
}

pub struct TileCoord {
    pub x:    u32,
    pub y:    u32,
    pub zoom: u8,
}

pub enum TileData {
    Image(ImageData),           // satellite imagery — RGBA bytes
    Terrain(TerrainData),       // elevation — Mapbox terrain-RGB or quantized mesh
    Vector(VectorTileData),     // Mapbox Vector Tiles
}
```

**Built-in tile sources:**

```rust
// XYZ tile source (OpenStreetMap, Mapbox, custom)
let imagery = XyzTileSource::new("https://tiles.example.com/{z}/{x}/{y}.png");

// Mapbox Raster Tiles
let imagery = MapboxTileSource::new(MapboxTileConfig {
    style:     MapboxStyle::SatelliteStreets,
    token:     "pk.eyJ1...",
    tile_size: 512,
});

// Mapbox Terrain-RGB
let terrain = MapboxTerrainSource::new("pk.eyJ1...");

// Local tile cache (file system)
let cache = LocalTileCache::new("/path/to/tiles/{z}/{x}/{y}.png");

// Custom tile source
let custom = MyTileSource::new(…);
renderer.add_tile_source(custom);
```

**Terrain mesh generation:**

Terrain is rendered as a subdivided quad mesh where vertex Y positions are displaced by elevation data from terrain tiles. Elevation is sampled from Mapbox terrain-RGB (encoding: height = -10000 + ((R × 256 × 256 + G × 256 + B) × 0.1)).

```rust
let terrain = TerrainRenderer::new(TerrainConfig {
    imagery_source: imagery_tile_source,
    elevation_source: terrain_tile_source,
    tile_size:        256,
    max_zoom:         18,
    cache_size_mb:    512,
    lod_levels:       8,
    skirt_depth:      100.0,   // prevents tile edge gaps
})?;
scene.add_terrain(terrain);
```

**Tile cache:**

LRU texture cache on GPU. Tiles are streamed asynchronously — visible but not-yet-loaded tiles show the parent tile texture at lower resolution until the higher-resolution tile arrives. No blank tiles, no pop-in.

```
Tile request pipeline:
  Visible tile at zoom N not in cache
    → Render parent tile at zoom N-1 (always available)
    → Async fetch tile at zoom N (HTTP or file)
    → Decode + upload to GPU texture cache
    → Replace parent tile rendering on next frame
```

**WGS84 coordinate system:**

Terrain positions are specified in WGS84 (latitude, longitude, altitude). Penumbra converts internally to a local cartesian coordinate system centered on the scene's geographic origin. Double-precision (`DVec3`) is used for geographic coordinates to avoid floating-point precision artifacts at high zoom levels.

```rust
pub struct GeoPosition {
    pub lat: f64,   // degrees
    pub lon: f64,   // degrees
    pub alt: f64,   // meters above WGS84 ellipsoid
}

// Convert geographic position to scene-local cartesian
let local_pos = scene.geo_to_local(GeoPosition { lat: 41.23, lon: -85.85, alt: 280.0 });
```

---

### 5.8 penumbra-atmosphere

Physically-based atmospheric scattering sky model. Bruneton-Neyret precomputed atmospheric scattering (the same model used in Google Earth and many AAA titles).

**Features:**
- Accurate Rayleigh and Mie scattering
- Sun disk rendering
- Moon rendering with phase
- Star field (procedural, correct stellar positions)
- Aerial perspective (distant objects appear hazier, bluer)
- Multiple time-of-day presets
- Weather: clear, overcast, fog, haze

```rust
let atmosphere = AtmosphereRenderer::new(AtmosphereConfig {
    sun_direction:    Vec3::new(-0.5, -0.8, -0.3).normalize(),
    sun_intensity:    1.0,
    rayleigh:         RayleighConfig::earth_default(),
    mie:              MieConfig::earth_default(),
    ground_albedo:    Rgb::new(0.1, 0.1, 0.1),
    exposure:         1.0,
})?;

scene.set_atmosphere(atmosphere);

// Animate time of day
atmosphere.set_sun_elevation(sun_elevation_degrees);
```

**Fog:**

```rust
scene.set_fog(Fog {
    mode:    FogMode::Exponential,
    color:   Rgb::new(0.7, 0.75, 0.8),
    density: 0.0002,
    start:   100.0,
    end:     50000.0,
});
```

---

### 5.9 penumbra-post

Post-processing pipeline. Composable passes applied after main 3D render, before final output.

**Built-in passes:**

| Pass | Description | Quality impact |
|------|-------------|----------------|
| `ToneMapping` | HDR → LDR (ACES, Reinhard, Uncharted2) | Required for HDR |
| `Bloom` | Luminance threshold + blur | High |
| `SSAO` | Screen-space ambient occlusion | High |
| `FXAA` | Fast approximate anti-aliasing | Medium |
| `TAA` | Temporal anti-aliasing | High |
| `DepthOfField` | Bokeh depth of field | Medium |
| `ColorGrading` | Lift/gamma/gain, saturation, contrast | Medium |
| `Vignette` | Edge darkening | Low |
| `ChromaticAberration` | Lens color fringing | Low |
| `Sharpen` | Edge sharpening | Low |
| `SMAA` | Subpixel morphological AA | High |

**Composable pipeline:**

```rust
renderer.set_post_pipeline(PostPipeline::new()
    .add(ToneMapping::aces())
    .add(Bloom::new(BloomConfig {
        threshold:  1.0,
        intensity:  0.15,
        radius:     0.005,
    }))
    .add(Ssao::new(SsaoConfig {
        radius:   0.5,
        bias:     0.025,
        power:    1.0,
        samples:  16,
    }))
    .add(Fxaa::default())
    .add(ColorGrading::new(ColorGradingConfig {
        exposure:    0.0,
        contrast:    1.05,
        saturation:  1.1,
        ..Default::default()
    }))
);
```

**Custom pass:**

```rust
struct MyCustomPass;

impl PostPass for MyCustomPass {
    fn render(
        &self,
        input:  &GpuTexture,
        output: &GpuTexture,
        depth:  &GpuTexture,
        frame:  &RenderFrame,
        backend: &mut dyn RenderBackend,
    ) { … }
}

renderer.set_post_pipeline(PostPipeline::new()
    .add(ToneMapping::aces())
    .add(MyCustomPass)
);
```

---

### 5.10 penumbra-shadow

Cascaded shadow maps for directional lights. PCF filtering for soft edges.

**Configuration:**

```rust
renderer.set_shadow_config(ShadowConfig {
    cascades:         4,
    map_size:         2048,    // per-cascade shadow map resolution
    pcf_samples:      16,
    pcf_radius:       1.5,
    max_distance:     500.0,   // shadow max distance from camera
    cascade_splits:   [0.05, 0.15, 0.4, 1.0],  // normalized cascade distances
    bias:             0.0005,
    normal_bias:      0.02,
});
```

**Point light shadows:**

Cube shadow maps for point lights:

```rust
Light::Point {
    cast_shadows: true,
    shadow_map_size: 512,
    shadow_near: 0.1,
    shadow_far:  50.0,
    ..
}
```

---

### 5.11 penumbra-text

Text rendering. Signed distance field (SDF) font rendering for crisp text at all scales.

**Font loading:**

```rust
let font = renderer.load_font(FontDescriptor {
    data:    include_bytes!("fonts/Inter-Regular.ttf"),
    name:    "Inter".to_string(),
    sdf:     true,
    atlas_size: 1024,
})?;
```

**3D text (world space):**

```rust
// Billboarded text that always faces the camera
scene.add_billboard_text(BillboardText {
    text:      "Entity Alpha".to_string(),
    font:      font_id,
    size:      14.0,
    color:     Rgba::WHITE,
    position:  Vec3::new(1.0, 2.0, 0.0),
    billboard: BillboardMode::ScreenAligned,
    max_width: Some(200.0),
});
```

**2D text (screen space):**

```rust
// HUD text
renderer.draw_text_2d(Text2d {
    text:     "60 FPS".to_string(),
    font:     font_id,
    size:     16.0,
    color:    Rgba::WHITE,
    position: Vec2::new(10.0, 10.0),
    anchor:   TextAnchor::TopLeft,
});
```

**Batched label rendering:**

For 27K entity labels: text is instanced using an SDF atlas. All visible labels for a single font are rendered in 1–3 draw calls using glyph instancing.

---

### 5.12 penumbra-compute

Compute shader support. Provides abstractions for GPU compute workloads.

**Compute pipeline:**

```rust
let pipeline = renderer.create_compute_pipeline(ComputePipelineDescriptor {
    shader:      include_wgsl!("shaders/my_compute.wgsl"),
    entry_point: "main",
    bind_groups: &[…],
})?;

renderer.dispatch_compute(pipeline, &bind_groups, [x, y, z]);
```

**Built-in compute passes:**

- GPU frustum culling (used by `penumbra-instance`)
- Tile texture decompression (used by `penumbra-terrain`)
- Atmosphere LUT precomputation (used by `penumbra-atmosphere`)
- SSAO sampling (used by `penumbra-post`)

---

### 5.13 penumbra-geo

Geospatial utilities. WGS84 coordinate system, geodesic math, projection systems.

**Coordinate conversions:**

```rust
use penumbra_geo::*;

// WGS84 ↔ ECEF (Earth-Centered, Earth-Fixed)
let ecef = geo::wgs84_to_ecef(GeoPosition { lat: 41.23, lon: -85.85, alt: 280.0 });
let wgs84 = geo::ecef_to_wgs84(ecef);

// WGS84 ↔ Local ENU (East-North-Up) — scene coordinate system
let origin = GeoPosition { lat: 41.23, lon: -85.85, alt: 0.0 };
let local = geo::wgs84_to_enu(position, origin);

// MGRS
let mgrs = geo::wgs84_to_mgrs(lat, lon);

// Haversine distance
let dist = geo::haversine_distance(pos_a, pos_b);  // meters

// Bearing
let bearing = geo::bearing(from, to);  // degrees true north

// Great circle interpolation
let midpoint = geo::great_circle_interpolate(from, to, 0.5);
```

**Tile math:**

```rust
// XYZ tile at zoom level
let tile = geo::lat_lon_to_tile(lat, lon, zoom);

// Tile bounds
let bounds = geo::tile_bounds(tile);  // BBox { min_lat, max_lat, min_lon, max_lon }

// Tile resolution (meters per pixel)
let res = geo::tile_resolution(zoom, lat);
```

---

### 5.14 penumbra-immediate

Immediate mode rendering API. Per-frame draw calls, no retained state.

```rust
// Lines
renderer.draw_line(Vec3::ZERO, Vec3::X, Rgba::RED, 1.0);
renderer.draw_polyline(&points, Rgba::WHITE, 1.5);

// Shapes
renderer.draw_sphere(center, radius, Rgba::BLUE);
renderer.draw_box(min, max, Rgba::GREEN);
renderer.draw_cylinder(base, top, radius, Rgba::YELLOW);
renderer.draw_cone(apex, base, radius, Rgba::ORANGE);
renderer.draw_frustum(camera, Rgba::CYAN);
renderer.draw_aabb(aabb, Rgba::WHITE);

// Arrows
renderer.draw_arrow(from, to, Rgba::RED, ArrowConfig { head_size: 0.1 });

// Billboards (icon rendering)
renderer.draw_billboard(BillboardDesc {
    position: Vec3::new(1.0, 2.0, 0.0),
    texture:  icon_texture,
    size:     Vec2::new(32.0, 32.0),   // screen pixels
    color:    Rgba::WHITE,
    pivot:    Vec2::new(0.5, 0.5),     // center
});

// Filled primitives
renderer.draw_filled_circle(center, radius, Rgba::new(1.0, 0.0, 0.0, 0.5));
renderer.draw_filled_rect(min, max, Rgba::new(0.0, 0.0, 1.0, 0.3));

// Grid (debug)
renderer.draw_grid(GridConfig {
    spacing: 1.0,
    count:   20,
    color:   Rgba::new(0.5, 0.5, 0.5, 0.3),
    y:       0.0,
});
```

All immediate mode draws are batched per frame — lines with lines, billboards with billboards — minimizing draw calls regardless of how many individual calls are made.

---

### 5.15 penumbra-camera

Camera system. Multiple camera types, smooth controls.

**Camera types:**

```rust
pub enum Camera {
    Perspective(PerspectiveCamera),
    Orthographic(OrthographicCamera),
}

pub struct PerspectiveCamera {
    pub position:   Vec3,
    pub target:     Vec3,
    pub up:         Vec3,
    pub fov_y:      f32,     // vertical FOV in degrees
    pub near:       f32,
    pub far:        f32,
    pub aspect:     f32,     // auto-set from viewport
}

pub struct OrthographicCamera {
    pub position:   Vec3,
    pub target:     Vec3,
    pub up:         Vec3,
    pub left:       f32,
    pub right:      f32,
    pub bottom:     f32,
    pub top:        f32,
    pub near:       f32,
    pub far:        f32,
}
```

**Camera controllers:**

```rust
// Orbit controller (click + drag to orbit, scroll to zoom)
let mut orbit = OrbitController::new(OrbitConfig {
    target:       Vec3::ZERO,
    distance:     10.0,
    min_distance: 0.1,
    max_distance: 10000.0,
    sensitivity:  1.0,
    invert_y:     false,
});

// Fly controller (WASD + mouse look)
let mut fly = FlyController::new(FlyConfig {
    speed:       5.0,
    sensitivity: 0.1,
    sprint_mult: 3.0,
});

// Globe controller (orbit around WGS84 globe)
let mut globe = GlobeController::new(GlobeConfig {
    min_altitude:  100.0,       // meters
    max_altitude:  20_000_000.0, // meters (from orbit down to street level)
    tilt_range:    (0.0, 90.0),
    sensitivity:   1.0,
});

// Update from input events
orbit.handle_mouse_move(dx, dy);
orbit.handle_scroll(delta);
let camera = orbit.camera();
```

**Raycasting / picking:**

```rust
// Screen position to world ray
let ray = camera.screen_to_ray(Vec2::new(x, y), viewport_size);

// Ray-mesh intersection
if let Some(hit) = scene.raycast(ray, RaycastConfig::default()) {
    println!("Hit node {:?} at distance {}", hit.node, hit.distance);
}

// Ray-terrain intersection (for globe click → geo coordinate)
if let Some(geo_pos) = terrain.raycast_geo(ray) {
    println!("Clicked at lat={} lon={}", geo_pos.lat, geo_pos.lon);
}
```

---

### 5.16 penumbra-asset

Asset loading pipeline. glTF 2.0, OBJ, image formats, HDR.

**Supported formats:**

| Format | Type | Notes |
|--------|------|-------|
| glTF 2.0 (.gltf, .glb) | 3D scene | Full PBR materials, animations, skins |
| OBJ + MTL | 3D mesh | Basic material support |
| PNG, JPEG, WebP | Texture | RGBA, RGB |
| KTX2 + Basis Universal | Texture | GPU-compressed, universal format |
| HDR (.hdr, .exr) | Environment | For IBL sky |
| TTF, OTF | Font | SDF generation at load time |

**Loading API:**

```rust
// Async asset loading
let gltf = renderer.load_gltf("models/vehicle.glb").await?;
let mesh  = gltf.meshes[0];
let mat   = gltf.materials[0];

// Synchronous (blocking)
let texture = renderer.load_texture_sync("textures/albedo.png")?;

// From bytes (no filesystem)
let mesh = renderer.load_gltf_bytes(include_bytes!("models/icon.glb")).await?;
```

---

### 5.17 penumbra-winit

Optional winit integration. Bridges winit window + event loop to Penumbra's renderer.

```rust
use penumbra_winit::PenumbraApp;

struct MyApp {
    scene: Scene,
    camera: OrbitController,
}

impl PenumbraApp for MyApp {
    fn init(renderer: &mut Renderer) -> Self { … }

    fn update(&mut self, dt: f32, input: &InputState) {
        self.camera.handle_input(input);
    }

    fn render(&mut self, frame: &mut RenderFrame, renderer: &mut Renderer) {
        self.scene.render(self.camera.camera(), renderer);
    }
}

penumbra_winit::run::<MyApp>(WindowConfig {
    title:  "My 3D App".to_string(),
    width:  1280,
    height: 720,
    vsync:  true,
});
```

---

### 5.18 penumbra-web

WASM + browser target support. Handles WebGPU/WebGL2 surface creation, async asset loading via `fetch`, and web-specific event handling.

```rust
// web entry point
#[wasm_bindgen(start)]
pub async fn start() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap();

    let backend = WgpuBackend::from_canvas(&canvas, WgpuConfig::default()).await?;
    let mut renderer = Renderer::new(backend, RendererConfig::default());

    // rest of init …

    penumbra_web::run_loop(move |dt| {
        renderer.begin_frame();
        // render …
        renderer.end_frame();
    });
}
```

---

## 6. Render Backend System

The `RenderBackend` trait is the only coupling point between Penumbra and any specific GPU API. All feature crates (`penumbra-pbr`, `penumbra-terrain`, etc.) talk exclusively to this trait.

**Writing a custom backend:**

Any application that already uses a specific GPU API can write a backend adapter:

```rust
struct MyVulkanBackend { /* vulkan state */ }

impl RenderBackend for MyVulkanBackend {
    fn create_mesh(&mut self, desc: MeshDescriptor) -> Result<GpuMesh, BackendError> {
        // upload to vulkan buffers
    }
    // ... implement all trait methods
}

let renderer = Renderer::new(MyVulkanBackend::new(…), config);
// Now use all Penumbra features on your Vulkan backend
```

---

## 7. Scene Graph

See Section 5.4 for the full `Scene` API.

### 7.1 Scene Composition

Multiple scenes can be composed. Use cases: main 3D scene + HUD scene + separate UI scene.

```rust
let mut main_scene = Scene::new();
let mut hud_scene  = Scene::new();

// Render order: main scene first, HUD on top
renderer.render_scenes(&[
    (&main_scene, &main_camera),
    (&hud_scene,  &hud_camera),
]);
```

### 7.2 Scene Serialization

Scenes serialize to/from a Penumbra JSON format for save/load:

```rust
let json = scene.to_json()?;
let scene = Scene::from_json(&json, &mut renderer)?;
```

---

## 8. Immediate Mode API

See Section 5.14 for the full immediate mode API.

### 8.1 Mixing Scene Graph and Immediate Mode

Both are submitted to the same `RenderFrame` per frame:

```rust
let mut frame = renderer.begin_frame();

// Scene graph rendering
main_scene.render(&camera, &mut renderer);

// Immediate mode overlaid on top
renderer.draw_aabb(selected_entity_aabb, Rgba::YELLOW);
renderer.draw_line(entity_pos, entity_pos + velocity * 5.0, Rgba::GREEN, 1.5);
renderer.draw_text_2d(hud_text, …);

renderer.end_frame(frame);
```

---

## 9. PBR Pipeline

See Section 5.5 for the PBR implementation details.

### 9.1 Render Pipeline Stages

```
Geometry pass (G-buffer: albedo, normal, metallic-roughness, emissive, depth)
    ↓
SSAO pass (optional, reads depth + normals)
    ↓
Shadow map pass (directional: cascaded, point: cube map)
    ↓
Lighting pass (PBR BRDF + IBL + punctual lights, reads G-buffer + shadows + SSAO)
    ↓
Transparent pass (forward, sorted back-to-front)
    ↓
Atmosphere / sky pass (renders behind all geometry)
    ↓
Post-processing chain (tone mapping → bloom → FXAA/TAA → color grading → …)
    ↓
2D overlay pass (immediate mode 2D, HUD, text)
    ↓
Present
```

Deferred rendering for opaque geometry (single lighting pass regardless of light count). Forward rendering for transparent geometry and special materials.

---

## 10. Instanced Rendering

See Section 5.6 for the full instanced rendering specification.

### 10.1 Entity Tracking Pattern

For NexusPulse Tactical's 27K+ entity use case:

```rust
// One batch per entity type
let aircraft_batch = renderer.create_instance_batch(InstanceBatchDesc {
    mesh:     aircraft_cone_mesh,
    material: aircraft_material,
    capacity: 10_000,
})?;

let ground_batch = renderer.create_instance_batch(InstanceBatchDesc {
    mesh:     ground_icon_mesh,
    material: icon_atlas_material,
    capacity: 20_000,
})?;

// Every frame: update all instances
let aircraft_instances: Vec<InstanceData> = entities
    .iter()
    .filter(|e| e.kind == EntityKind::Aircraft)
    .map(|e| InstanceData {
        transform: Mat4::from_translation(e.scene_pos) * Mat4::from_quat(e.heading_quat),
        color:     e.track_color,
        uv_offset: e.icon_atlas_uv_offset,
        uv_scale:  e.icon_atlas_uv_scale,
        user_data: [e.altitude, e.speed, e.threat_level, 0.0],
    })
    .collect();

renderer.update_instances(aircraft_batch, &aircraft_instances)?;
```

---

## 11. Tile-Based Streaming

See Section 5.7 for the full tile streaming specification.

---

## 12. Atmosphere & Sky

See Section 5.8 for the full atmosphere specification.

---

## 13. Post-Processing Pipeline

See Section 5.9 for the full post-processing specification.

---

## 14. Shadow System

See Section 5.10 for the full shadow specification.

---

## 15. Text Rendering

See Section 5.11 for the full text rendering specification.

---

## 16. 2D + 3D Compositing

2D and 3D content are composited in the same frame via the render pipeline stage ordering (Section 9.1). The 2D overlay pass renders on top of 3D content using screen-space coordinates. Depth testing is disabled for 2D overlay content.

Z-ordering within 2D content is explicit — draw order is submission order.

---

## 17. Compute Shaders

See Section 5.12 for the compute shader specification.

### 17.1 Use Cases

- **GPU frustum culling** — runs as compute before draw calls, produces indirect draw buffer
- **Terrain tile decompression** — decompress compressed tile formats on GPU
- **Atmosphere LUT precomputation** — compute scattering lookup tables once at init
- **Particle simulation** — compute-based particle systems
- **Custom application compute** — expose to SDK consumers for their own workloads

---

## 18. Camera System

See Section 5.15 for the full camera specification.

---

## 19. Asset Pipeline

See Section 5.16 for the full asset loading specification.

---

## 20. WASM / Browser Target

See Section 5.18 for the full WASM target specification.

### 20.1 Feature Parity Matrix

| Feature | Native (wgpu) | WebGPU | WebGL2 |
|---------|--------------|--------|--------|
| PBR | ✓ | ✓ | ✓ |
| Instancing | ✓ | ✓ | ✓ |
| GPU frustum culling | ✓ | ✓ | CPU fallback |
| Tile streaming | ✓ | ✓ | ✓ |
| Atmosphere | ✓ | ✓ | ✓ |
| SSAO | ✓ | ✓ | ✓ |
| Bloom | ✓ | ✓ | ✓ |
| TAA | ✓ | ✓ | ✗ (FXAA fallback) |
| Compute shaders | ✓ | ✓ | CPU fallback |
| Shadow maps | ✓ | ✓ | ✓ |
| HDR textures | ✓ | ✓ | ✓ |
| KTX2 compressed textures | ✓ | ✓ | Partial |

---

## 21. NexusPulse Tactical Integration

NexusPulse Tactical uses Penumbra as its 3D globe rendering engine. The integration is a first-class use case that drives several design decisions:

**Globe view requirements:**
- WGS84 globe with satellite imagery (Mapbox or custom tile source)
- Terrain elevation at operational zoom levels
- 27K+ CoT entities rendered as instanced icons/cones with track history trails
- Real-time entity position updates at 1Hz (WebSocket from tactical-server)
- Entity selection via mouse click (raycasting)
- Globe orbit/zoom/tilt camera (GlobeController)
- Atmosphere rendering for realistic sky
- 2D HUD overlay (entity count, connection status, threat summary)
- Post-processing: tone mapping + FXAA + color grading
- Runs in browser (WASM + WebGPU/WebGL2) AND native desktop (Tauri)

**Integration crate:**

```toml
[dependencies]
penumbra          = { version = "1", features = ["geo", "terrain", "atmosphere"] }
penumbra-instance = "1"
nexuspulse-tactical = { path = "../NexusPulse-Tactical" }
```

---

## 22. Performance Targets

These are hard requirements, not aspirational goals:

| Scenario | Target | Hardware |
|----------|--------|----------|
| 27K instanced entities, frustum culled | 60fps | GTX 1060 / M1 |
| Full globe view, zoom 12, satellite tiles | 60fps | GTX 1060 / M1 |
| Atmosphere + terrain + 27K entities + post | 60fps | GTX 1060 / M1 |
| Same scene in browser (WebGPU) | 60fps | Chrome on M1 |
| Same scene in browser (WebGL2 fallback) | 30fps | Chrome on Intel UHD |
| Initial tile load (cold cache, zoom 8) | < 3s to first render | — |
| Frame time (GPU), 27K entities | < 8ms | GTX 1060 |
| Memory (GPU), 27K entities + terrain | < 1.5GB | — |

**How 60fps at 27K entities is achieved:**
1. GPU-side frustum culling via compute — visible entities only reach the draw stage
2. Single instanced draw call per entity type (typically 2–4 draw calls for all entities)
3. Instance buffer updated with `write_buffer` — single CPU→GPU transfer per frame
4. SDF text labels batched — single draw call for all visible labels
5. Tile cache prevents redundant texture uploads — tiles stay on GPU until evicted

---

## 23. Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Core language | Rust 2024 | Performance, safety, WASM |
| Default GPU backend | wgpu 0.20+ | Vulkan/Metal/DX12/WebGPU/WebGL2 |
| Shader language | WGSL | Native wgpu, cross-platform via Naga |
| Math | glam | Fast, SIMD, widely used in Rust 3D |
| Double precision geo | glam DVec3 | WGS84 precision |
| glTF loading | gltf crate | Industry standard 3D format |
| Image loading | image crate | PNG/JPEG/WebP |
| HDR loading | exr + radiance | EXR and HDR formats |
| KTX2 loading | ktx2 crate | GPU-compressed textures |
| Font/SDF | ab_glyph + custom SDF | TTF to SDF atlas |
| Tile fetching | reqwest (native) / web-sys fetch (WASM) | Async HTTP |
| Tile cache | Custom LRU | GPU texture LRU |
| Async runtime | tokio (native) / wasm-bindgen-futures (WASM) | Tile streaming |
| Window integration | winit | Cross-platform windowing |
| WASM target | wasm-bindgen + web-sys | Browser integration |
| Serialization | serde + serde_json | Scene serialization |
| Logging | tracing | Structured |
| Proc macros | syn + quote | Shader variant generation |

---

## 24. Directory Structure

```
Penumbra/
├── Cargo.toml                    # workspace manifest
├── Cargo.lock
├── README.md
├── PROGRESS.md                   # live progress tracker — always current
├── ARCHITECTURE.md
├── PERFORMANCE.md                # performance targets, benchmark results
├── CHANGELOG.md
├── LICENSE                       # MIT OR Apache-2.0
│
├── crates/
│   ├── penumbra-core/            # Renderer, RenderFrame, core types
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   ├── penumbra-backend/         # RenderBackend trait
│   ├── penumbra-wgpu/            # wgpu backend implementation
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── surface.rs
│   │       ├── pipeline.rs
│   │       ├── buffer.rs
│   │       ├── texture.rs
│   │       └── pass.rs
│   ├── penumbra-scene/           # Scene graph
│   ├── penumbra-pbr/             # PBR pipeline
│   │   └── src/
│   │       └── shaders/          # WGSL shaders
│   │           ├── pbr.wgsl
│   │           ├── gbuffer.wgsl
│   │           └── lighting.wgsl
│   ├── penumbra-instance/        # Instanced rendering
│   ├── penumbra-terrain/         # Tile streaming + terrain
│   ├── penumbra-atmosphere/      # Sky + fog
│   ├── penumbra-post/            # Post-processing
│   ├── penumbra-shadow/          # Shadow maps
│   ├── penumbra-text/            # SDF text
│   ├── penumbra-compute/         # Compute shaders
│   ├── penumbra-geo/             # WGS84 geodesy
│   ├── penumbra-immediate/       # Immediate mode
│   ├── penumbra-camera/          # Camera + controllers
│   ├── penumbra-asset/           # Asset loading
│   ├── penumbra-winit/           # winit integration
│   └── penumbra-web/             # WASM / browser target
│
├── examples/
│   ├── hello_cube/               # simplest possible 3D
│   ├── pbr_scene/                # PBR materials + IBL
│   ├── instanced_entities/       # 27K entity instancing
│   ├── globe/                    # full globe with satellite tiles + terrain
│   ├── atmosphere/               # sky + fog
│   ├── post_processing/          # full post pipeline
│   ├── text_labels/              # SDF text in 3D
│   ├── immediate_mode/           # immediate mode API demo
│   ├── tactical_globe/           # NexusPulse Tactical demo
│   └── wasm/                     # WASM browser example
│
└── benches/
    ├── instancing.rs             # 27K entity throughput
    ├── tile_streaming.rs         # tile load + render perf
    └── full_scene.rs             # complete tactical globe scenario
```

---

## 25. Development Phases

### Phase 1 — Core + wgpu Backend + PBR + Scene Graph (Weeks 1–8)

- `penumbra-core`: all shared types, `Renderer`, `RenderFrame`
- `penumbra-backend`: `RenderBackend` trait
- `penumbra-wgpu`: full wgpu backend (native targets)
- `penumbra-scene`: scene graph, transform hierarchy, frustum culling
- `penumbra-pbr`: deferred PBR pipeline, IBL, directional + point + spot lights
- `penumbra-camera`: perspective/ortho cameras, orbit + fly controllers
- `penumbra-asset`: glTF 2.0 + OBJ + PNG/JPEG loading
- Example: `hello_cube`, `pbr_scene`

End state: Load a glTF model, render it with PBR lighting in a scene graph, orbit camera, 60fps on target hardware.

### Phase 2 — Instancing + Shadow + Post + Immediate (Weeks 9–14)

- `penumbra-instance`: instanced rendering + GPU frustum culling
- `penumbra-shadow`: cascaded shadow maps + PCF
- `penumbra-post`: tone mapping, bloom, SSAO, FXAA, color grading
- `penumbra-immediate`: full immediate mode draw API
- `penumbra-compute`: compute pipeline abstraction
- Example: `instanced_entities` (27K entities at 60fps verified)

End state: 27K instanced entities at 60fps with shadows and post-processing. Immediate mode overlay working alongside scene graph.

### Phase 3 — Terrain + Atmosphere + Text + Geo (Weeks 15–21)

- `penumbra-terrain`: tile streaming (XYZ + Mapbox), terrain mesh generation, LRU tile cache
- `penumbra-atmosphere`: Bruneton-Neyret scattering, sun/moon/stars, fog
- `penumbra-text`: SDF font rendering, billboard text, batched labels
- `penumbra-geo`: WGS84 geodesy, MGRS, tile math, haversine
- `penumbra-camera`: GlobeController
- Example: `globe` (full globe with satellite imagery + terrain), `tactical_globe`

End state: Full globe view with satellite tiles, terrain elevation, atmosphere, 27K entity labels. Raycasting for globe click → geo coordinate.

### Phase 4 — WASM + Browser + Polish (Weeks 22–26)

- `penumbra-wgpu`: WASM target (WebGPU + WebGL2 fallback)
- `penumbra-web`: browser surface creation, fetch tile loading
- `penumbra-winit`: winit integration crate
- All examples ported to WASM
- Benchmarks run and PERFORMANCE.md populated
- crates.io preparation (all `publish = false` until authorized)

End state: Full feature set running in browser at target frame rates. All examples working native + WASM. Benchmarks documented.

---

## 26. Test Strategy

### Unit Tests

- `penumbra-geo`: all coordinate conversions correct (WGS84↔ECEF↔ENU, MGRS, tile math)
- `penumbra-scene`: transform propagation correct for deep hierarchies; frustum culling correct
- `penumbra-camera`: view/projection matrices correct; raycasting correct
- `penumbra-terrain`: tile coord math correct; LRU cache evicts correctly
- `penumbra-post`: pipeline stage ordering correct

### Integration Tests

- `penumbra-wgpu` headless: render a frame, read back pixels, verify expected output
- PBR: known light setup produces known pixel output (within tolerance)
- Shadow: shadow-receiving surface produces expected shadow pixels
- Instancing: 27K instances render without GPU errors

### Performance Benchmarks (criterion)

```bash
cargo bench -p penumbra-instance  # 27K entity throughput
cargo bench -p penumbra-terrain   # tile streaming bandwidth
cargo bench -p penumbra           # full scene composite
```

Benchmarks run in CI on GPU-equipped runner. Regression > 5% fails CI.

### WASM Tests

```bash
wasm-pack test --headless --chrome crates/penumbra-geo
wasm-pack test --headless --chrome crates/penumbra-core
```

---

## 27. Acceptance Criteria

### Phase 1 Complete When:

- [ ] Load and render a glTF 2.0 model with PBR materials at 60fps
- [ ] Scene graph transform hierarchy correct to 10 levels deep
- [ ] Frustum culling removes off-screen nodes from draw calls
- [ ] IBL sky renders correctly with provided HDR environment
- [ ] Orbit camera correctly orbits around scene origin
- [ ] glTF asset with multiple meshes and materials loads without error
- [ ] All unit tests pass

### Phase 2 Complete When:

- [ ] 27,000 instanced entities render at ≥ 60fps on GTX 1060 equivalent
- [ ] GPU frustum culling reduces draw calls to visible instances only
- [ ] Cascaded shadow maps render correctly for directional light
- [ ] Full post-processing chain (tone map + bloom + SSAO + FXAA) runs without error
- [ ] Immediate mode lines, shapes, and billboards render correctly alongside scene graph
- [ ] Criterion benchmark: 27K instances < 8ms GPU frame time

### Phase 3 Complete When:

- [ ] Satellite tile streaming loads XYZ tiles from a live tile server
- [ ] Terrain elevation displaces mesh correctly from Mapbox terrain-RGB
- [ ] Tile cache LRU eviction works correctly under memory pressure
- [ ] Atmosphere renders correct Rayleigh scattering at multiple sun elevations
- [ ] SDF text renders crisply at all scales
- [ ] 27K entity labels render in ≤ 3 draw calls
- [ ] Globe raycast returns correct WGS84 coordinate on click
- [ ] GlobeController smoothly animates from orbit altitude to street level

### Phase 4 Complete When:

- [ ] Full feature set compiles to WASM without errors
- [ ] Globe example runs at ≥ 60fps in Chrome (WebGPU)
- [ ] Globe example runs at ≥ 30fps in Chrome (WebGL2 fallback)
- [ ] All Phase 1–3 examples run identically in WASM
- [ ] PERFORMANCE.md populated with benchmark results
- [ ] All tests pass

---

## 28. Claude Code Operational Instructions

### 28.1 Repository

Create `github.com/AutomataNexus/Penumbra` as a **private** repository. Do NOT make it public until explicitly instructed. Do NOT publish any crates to crates.io until explicitly instructed. Set `publish = false` in every `Cargo.toml`.

**About:** `General-purpose 3D rendering SDK for Rust. PBR, instanced entities, satellite tile streaming, atmosphere, post-processing, WASM. The Three.js of the Rust ecosystem.`

**Topics:** `rust, 3d, rendering, wgpu, webgpu, wasm, pbr, geospatial, globe, cesium, three-js, graphics, game-dev`

### 28.2 Documentation — Create Before Code

Create all documentation files **before writing any implementation code.**

**Required files — create first:**
- `README.md` — full project README with ASCII architecture and data flow diagrams
- `PROGRESS.md` — progress tracker, all Phase 1 checkboxes pre-populated unchecked
- `ARCHITECTURE.md` — architecture document covering render pipeline stages, backend trait design, scene graph design, immediate mode design
- `PERFORMANCE.md` — performance targets table (from Section 22), benchmark methodology, results (populated as benchmarks run)
- `CHANGELOG.md` — empty, ready

**PROGRESS.md format:**

```markdown
# Penumbra — Progress Tracker
Last updated: YYYY-MM-DD HH:MM UTC

## Current Phase
Phase 1 — Core + wgpu + PBR + Scene Graph

## Overall Progress
[██░░░░░░░░░░░░░░░░░░] 10%

## Phase 1 Checklist

### penumbra-core
- [x] Renderer struct
- [x] RenderFrame struct
- [x] RendererConfig, FrameStats
- [ ] GpuMesh, GpuTexture, GpuBuffer types (in progress)
- [ ] Material, AlphaMode
- [ ] Aabb, DrawCall
- [ ] Unit tests

### penumbra-backend
- [ ] Not started

### penumbra-wgpu
- [ ] Not started
...

## Phase 2 Checklist
(all unchecked — not started)

## Phase 3 Checklist
(all unchecked — not started)

## Phase 4 Checklist
(all unchecked — not started)

## Performance Benchmark Results
| Scenario | Target | Measured | Status |
|----------|--------|----------|--------|
| 27K entities @60fps | ✓ | — | pending |

## Recent Commits
- 2026-03-30 feat(penumbra-core): Renderer + RenderFrame types

## Blockers
- None

## Next
- Complete GpuMesh, GpuTexture types in penumbra-core
- Begin penumbra-backend RenderBackend trait
```

**Per-crate README format:**

Every crate has a README. Created when crate is scaffolded. Required sections:
1. Purpose — one sentence
2. Position in pipeline — ASCII diagram
3. Key public types (3–5 items)
4. Usage example — compiles, no `todo!()`
5. How to run tests

### 28.3 Build Order

```
1.  penumbra-core      (foundation types, Renderer)
2.  penumbra-backend   (RenderBackend trait)
3.  penumbra-wgpu      (default backend)
4.  penumbra-geo       (geodesy, no rendering dep)
5.  penumbra-camera    (depends: penumbra-core)
6.  penumbra-scene     (depends: penumbra-core, penumbra-camera)
7.  penumbra-compute   (depends: penumbra-backend)
8.  penumbra-pbr       (depends: penumbra-core, penumbra-backend)
9.  penumbra-shadow    (depends: penumbra-core, penumbra-pbr)
10. penumbra-instance  (depends: penumbra-core, penumbra-compute)
11. penumbra-terrain   (depends: penumbra-core, penumbra-geo)
12. penumbra-atmosphere(depends: penumbra-core, penumbra-compute)
13. penumbra-post      (depends: penumbra-core, penumbra-backend)
14. penumbra-text      (depends: penumbra-core)
15. penumbra-immediate (depends: penumbra-core)
16. penumbra-asset     (depends: penumbra-core)
17. penumbra-winit     (depends: penumbra-core, penumbra-wgpu)
18. penumbra-web       (depends: penumbra-core, penumbra-wgpu)
```

### 28.4 Shader Discipline

- All shaders written in WGSL — no GLSL, no HLSL, no SPIR-V hand-coding
- Shaders live in `crates/{crate}/src/shaders/` directory
- Every shader has a corresponding Rust integration test (headless render + pixel readback)
- No shader compilation at runtime — shaders are compiled to pipeline objects at init time
- WebGL2 compatibility: no features beyond WebGL2 limits (check wgpu's `Limits::downlevel_webgl2_defaults()`)

### 28.5 Performance Discipline

- Run `cargo bench` after completing each phase
- Populate PERFORMANCE.md benchmark results table after each benchmark run
- Any benchmark regression > 5% from documented target requires investigation before proceeding
- 27K entity benchmark must pass before Phase 2 is marked complete — this is a hard gate

### 28.6 Code Quality

- Zero `todo!()` or `unimplemented!()` in implementation code
- Zero mock/stub implementations
- `cargo clippy -- -D warnings` must pass on every commit
- `cargo audit` must pass on every commit
- All crates have unit tests
- `unsafe` only in `penumbra-wgpu` where wgpu requires it — never in feature crates

### 28.7 Example Discipline

Every phase ships working examples. Examples are not toy demos — they are realistic demonstrations of the phase's features:

- `hello_cube` — must use real PBR materials, not a flat color cube
- `instanced_entities` — must actually render 27K instances, not 100
- `globe` — must load real satellite tiles from a live tile server, not a static texture
- All examples compile and run on the first `cargo run --example <name>`

### 28.8 Commit Discipline

- Commit after every completed unit of work
- Format: `feat(crate): description` / `fix(crate): description` / `perf(crate): description` / `shader(crate): description` / `docs: description`
- Update PROGRESS.md in the same commit
- Push after every commit

### 28.9 Do Not

- Do not publish any crate to crates.io — `publish = false` in all `Cargo.toml`
- Do not make the repository public until instructed
- Do not write GLSL or HLSL shaders — WGSL only
- Do not depend on Bevy — Penumbra is not built on or compatible with Bevy
- Do not add an ECS — Penumbra has no entity-component system
- Do not add an event loop — that is the application's responsibility
- Do not add audio, physics, input, or networking — pure rendering only
- Do not write implementation code before the crate's README.md exists
- Do not commit without updating PROGRESS.md

---

*Penumbra — The Three.js of the Rust ecosystem.*
*Andrew Jewell Sr. — AutomataNexus LLC — devops@automatanexus.com*
