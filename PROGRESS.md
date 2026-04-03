# Penumbra -- Progress Tracker
Last updated: 2026-04-03 08:00 UTC

## Current Phase
All phases complete. Audit findings resolved. Remaining: GPU hardware benchmarks.

## Overall Progress
[███████████████████░] 95%

## Phase 1 -- Core + wgpu + PBR + Scene Graph

### penumbra-core
- [x] Renderer struct
- [x] RenderFrame struct
- [x] RendererConfig, FrameStats
- [x] GpuMesh, GpuTexture, GpuBuffer types
- [x] Material, AlphaMode
- [x] Aabb, DrawCall
- [x] Math type re-exports (glam)
- [x] Unit tests

### penumbra-backend
- [x] RenderBackend trait
- [x] BackendCapabilities
- [x] Handle types (MeshId, TextureId, BufferId, PipelineId)
- [x] Descriptor types (Mesh, Texture, Buffer, Pipeline, ComputePipeline)
- [x] RenderPassHandle, ComputePassHandle
- [x] WASM-compatible Send/Sync via MaybeSend/MaybeSync
- [x] BindGroupEntry, BindGroupLayoutDescriptor
- [x] Color types (Rgba, Rgb) with serde
- [x] Unit tests

### penumbra-wgpu
- [x] WgpuBackend struct
- [x] Headless surface creation
- [x] Resource creation (mesh, texture, buffer, pipeline, compute pipeline)
- [x] Full bind group creation with layout
- [x] Frame lifecycle (begin_frame clears state, end_frame polls device)
- [x] Render pass: recorded commands replayed through real wgpu RenderPass
- [x] Compute pass: recorded commands replayed through real wgpu ComputePass
- [x] Type conversion (Penumbra <-> wgpu, all formats)
- [x] WASM compilation (wasm32-unknown-unknown)

### penumbra-geo
- [x] GeoPosition type
- [x] WGS84 to ECEF conversion
- [x] ECEF to WGS84 conversion (iterative Bowring)
- [x] WGS84 to ENU conversion
- [x] ENU to WGS84 conversion
- [x] Haversine distance
- [x] Bearing calculation
- [x] Great circle interpolation
- [x] Tile math (lat/lon to tile, tile bounds, tile resolution)
- [x] Unit tests (22 passing)

### penumbra-camera
- [x] PerspectiveCamera
- [x] OrthographicCamera
- [x] Camera enum (unified)
- [x] OrbitController
- [x] FlyController
- [x] GlobeController (WGS84 orbit, altitude zoom, tilt, heading)
- [x] View/projection matrix computation
- [x] Screen-to-ray raycasting
- [x] Ray-plane and ray-AABB intersection
- [x] Unit tests (18 passing)

### penumbra-scene
- [x] Scene struct with SlotMap
- [x] SceneNode, Transform (translation, rotation, scale)
- [x] Renderable enum (Mesh, Light)
- [x] Light enum (Directional, Point, Spot)
- [x] Parent/child hierarchy (set_parent, remove_node)
- [x] Transform hierarchy propagation (lazy, root-to-leaf)
- [x] Frustum culling (6-plane Gribb/Hartmann, AABB test)
- [x] LOD system (screen-size threshold selection)
- [x] Unit tests (deep hierarchy, culling, LOD)

### penumbra-pbr
- [x] Light enum (Directional, Point, Spot)
- [x] LightUniform (Pod, 64 bytes)
- [x] MaterialUniform (Pod, 48 bytes)
- [x] PbrConfig, PbrPipeline (add/clear/query lights)
- [x] EnvironmentConfig (IBL)
- [x] WGSL shaders: pbr.wgsl (vertex + G-buffer), lighting.wgsl (Cook-Torrance BRDF)
- [x] Unit tests

### penumbra-asset
- [x] glTF 2.0 loading (meshes, materials, PBR properties)
- [x] OBJ loading (manual parser, fan triangulation)
- [x] Image loading (PNG, JPEG via image crate, RGBA8 output)
- [x] Primitive generators: cube (24v/36i), sphere (UV), plane (subdivided)
- [x] Unit tests

### Examples
- [x] hello_cube (PBR cube, orbit camera, 3 lights, scene graph)
- [x] pbr_scene (multiple materials, shadows, post-processing, hierarchy)

## Phase 2 -- Instancing + Shadow + Post + Immediate

### penumbra-instance
- [x] InstanceData (Pod, 96 bytes: transform + color + uv_offset + uv_scale)
- [x] InstanceBatch, InstanceBatchDesc, InstanceBatchId
- [x] InstanceManager (create/update/remove batches, capacity enforcement)
- [x] CPU frustum culling fallback (clip-space w-test)
- [x] Unit tests (size, batch ops, capacity, culling)

### penumbra-shadow
- [x] ShadowConfig (cascades, map_size, PCF, bias)
- [x] CascadeShadowMap (logarithmic split scheme, light-space matrices)
- [x] PointShadowMap (6-face cubemap projections)
- [x] ShadowUniform (Pod)
- [x] WGSL shader: shadow.wgsl (PCF sampling)
- [x] Unit tests

### penumbra-post
- [x] PostPass trait, PostPipeline (builder pattern)
- [x] ToneMapping (ACES, Reinhard, Uncharted2, Linear)
- [x] Bloom, Ssao, Fxaa, ColorGrading
- [x] Vignette, ChromaticAberration, Sharpen
- [x] WGSL shaders: tone_mapping.wgsl, fxaa.wgsl
- [x] Unit tests

### penumbra-immediate
- [x] ImmediateVertex (Pod), ImmediateBatch
- [x] ImmediateRenderer: draw_line, draw_polyline, draw_box, draw_sphere
- [x] draw_arrow, draw_aabb, draw_grid, draw_filled_rect
- [x] BillboardDesc
- [x] Unit tests

### penumbra-compute
- [x] ComputeTask, ComputeScheduler
- [x] GpuCulling with workgroup_count
- [x] WGSL shader: frustum_cull.wgsl
- [x] Unit tests

### Examples
- [x] instanced_entities (27K entities, CPU frustum culling, batch management)

## Phase 3 -- Terrain + Atmosphere + Text + Geo

### penumbra-terrain
- [x] TileCoord with parent/children
- [x] TileSource trait, XyzTileSource (URL template)
- [x] TileCache (LRU eviction)
- [x] Terrain mesh generation (subdivided quad, height displacement)
- [x] Terrain-RGB elevation decoding (Mapbox format)
- [x] TerrainConfig
- [x] Unit tests

### penumbra-atmosphere
- [x] AtmosphereConfig (earth_default: planet radius, atmosphere height)
- [x] RayleighConfig, MieConfig (earth_default)
- [x] Fog (Linear, Exponential, ExponentialSquared) with fog_factor()
- [x] AtmosphereRenderer (sun direction, sun elevation)
- [x] AtmosphereUniform (Pod)
- [x] WGSL shader: atmosphere.wgsl
- [x] Unit tests

### penumbra-text
- [x] FontAtlas, FontId, FontDescriptor, GlyphMetrics
- [x] TextLayout, PositionedGlyph, layout_text()
- [x] BillboardText, BillboardMode, Text2d, TextAnchor
- [x] GlyphVertex (Pod), TextBatch (add_layout, vertex/index generation)
- [x] WGSL shader: sdf_text.wgsl
- [x] Unit tests

### Examples
- [x] globe (WGS84 geodesy, tile math, terrain mesh, atmosphere, cache)
- [x] tactical_globe (27K entities + labels + HUD + globe + frustum culling)

## Phase 4 -- WASM + Browser + Winit + Benchmarks

### penumbra-wgpu (WASM)
- [x] Compiles to wasm32-unknown-unknown
- [x] MaybeSend/MaybeSync for WASM compatibility

### penumbra-web
- [x] WebConfig, WebPlatform, WebError types
- [x] Browser surface creation (create_surface)
- [x] Fetch-based tile loading (fetch_tile, fetch_tiles_async)
- [x] Render loop (run_loop via requestAnimationFrame)
- [x] WASM init (console_error_panic_hook)
- [x] Real wasm-bindgen + web-sys implementation (wasm.rs module)
- [x] Platform detection (WebGPU, WebGL2, user agent, DPI)
- [x] Compiles to wasm32-unknown-unknown
- [x] Unit tests (native stubs)

### penumbra-winit
- [x] WindowConfig (title, size, vsync, resizable)
- [x] InputState (mouse pos/delta, scroll, buttons, keys, per-frame reset)
- [x] KeyCode enum with winit key mapping
- [x] PenumbraApp trait (init, update, render, resize)
- [x] Full winit event loop (ApplicationHandler, create window, input dispatch)
- [x] Unit tests

### penumbra (root crate)
- [x] Re-exports all 18 sub-crates
- [x] Criterion benchmarks: instancing, tile_streaming, full_scene
- [x] Benchmarks compile

### WASM Compilation
- [x] All 16 non-winit/asset crates compile to wasm32-unknown-unknown
- [x] WASM example binary built with wasm-pack

### Examples
- [x] wasm (browser entry point with index.html, wasm-pack build)

## Audit Fixes (2026-04-03)
- [x] HIGH: begin_frame/end_frame return Result instead of panicking
- [x] HIGH: read_buffer does real GPU readback via staging buffer + map
- [x] MED: Material storage switched from Vec to HashMap (O(1) lookup)
- [x] MED: write_texture uses actual format bpp instead of hardcoded 4
- [x] LOW: MaterialId::INVALID sentinel for default Material
- [x] LOW: Aabb::from_points returns zero AABB for empty slice
- [x] LOW: lat_lon_to_tile clamps latitude to Web Mercator bounds
- [x] LOW: Ray AABB intersection guards against NaN
- [x] LOW: from_wgpu_texture_format warns on unknown format fallback
- [x] LOW: Spot light shadows field now written to uniform

## Performance Benchmark Results (CPU-side, no GPU)
| Scenario | Measured | Notes |
|----------|----------|-------|
| Scene update 100 nodes | 651 ns | Transform propagation |
| Scene update 1K nodes | 6.6 us | Transform propagation |
| Scene update 5K nodes | 34 us | Transform propagation |
| Full frame prep 27K | 41 us | Cull + lights + text layout |
| CPU frustum cull 27K | 42 us | Point-in-frustum clip test |
| Instance gen 27K | 218 us | Create InstanceData array |
| Batch update 27K | 96 us | Vec copy to batch |
| Tile cache insert 256 | 29 us | LRU insert |
| Tile cache lookup | 6.7 ns | HashMap get |
| Terrain mesh gen 32x32 | 5.0 us | Vertex + index generation |
| Terrain RGB decode 256x256 | 78 us | 65K pixel decode |

## GPU Benchmark Results
| Scenario | Target | Measured | Status |
|----------|--------|----------|--------|
| 27K entities @60fps | 60fps | -- | requires GPU hardware |
| Globe zoom 12 tiles | 60fps | -- | requires GPU hardware |
| Full scene composite | 60fps | -- | requires GPU hardware |
| WebGPU browser | 60fps | -- | requires GPU hardware |
| WebGL2 browser | 30fps | -- | requires GPU hardware |

## Blockers
- GPU rendering benchmarks require GPU-equipped runner

## Summary
- 19 crates (18 sub-crates + 1 root umbrella)
- 6 examples (hello_cube, pbr_scene, instanced_entities, globe, tactical_globe, wasm)
- 3 criterion benchmarks (instancing, tile_streaming, full_scene)
- 8 WGSL shaders
- 116 unit tests, all passing
- cargo clippy --workspace clean (zero warnings)
- Native + WASM compilation verified
- 10 audit findings resolved
