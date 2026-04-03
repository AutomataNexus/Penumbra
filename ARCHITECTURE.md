# Penumbra -- Architecture

## Overview

Penumbra is a general-purpose 3D rendering SDK for Rust, sitting between raw GPU bindings (wgpu) and full game engines (Bevy). It provides a complete rendering toolkit without imposing an application framework, ECS, event loop, or any non-rendering concern.

## Render Pipeline Stages

```
+-------------------+
| Geometry Pass     |  G-buffer: albedo, normal, metallic-roughness, emissive, depth
+--------+----------+
         |
+--------v----------+
| SSAO Pass         |  Optional, reads depth + normals
+--------+----------+
         |
+--------v----------+
| Shadow Map Pass   |  Directional: cascaded shadow maps; Point: cube maps
+--------+----------+
         |
+--------v----------+
| Lighting Pass     |  PBR BRDF (Cook-Torrance) + IBL + punctual lights
+--------+----------+  Reads G-buffer + shadow maps + SSAO
         |
+--------v----------+
| Transparent Pass  |  Forward rendering, sorted back-to-front
+--------+----------+
         |
+--------v----------+
| Atmosphere Pass   |  Bruneton-Neyret scattering, renders behind geometry
+--------+----------+
         |
+--------v----------+
| Post-Processing   |  Tone mapping -> Bloom -> FXAA/TAA -> Color grading
+--------+----------+
         |
+--------v----------+
| 2D Overlay Pass   |  Immediate mode 2D, HUD, text (no depth test)
+--------+----------+
         |
+--------v----------+
| Present           |  Swap chain present
+-------------------+
```

Deferred rendering for opaque geometry (single lighting pass regardless of light count). Forward rendering for transparent geometry and special materials.

## Backend Trait Design

The `RenderBackend` trait is the sole coupling point between Penumbra and any GPU API. All feature crates depend only on this trait, never on wgpu directly.

```
penumbra-pbr    --+
penumbra-terrain --+--> penumbra-backend::RenderBackend
penumbra-post   --+            |
penumbra-shadow --+    +-------+-------+
                       |               |
                 penumbra-wgpu   CustomBackend
```

The trait covers:
- **Resource creation/destruction** -- meshes, textures, buffers, pipelines
- **Buffer/texture updates** -- write_buffer, write_texture
- **Frame lifecycle** -- begin_frame, end_frame, present
- **Render pass** -- begin/end, set pipeline/bind group/buffers, draw/draw_indexed
- **Compute pass** -- begin/end, set pipeline, dispatch
- **Capabilities** -- max texture size, compute support, indirect draw support

## Scene Graph Design

SlotMap-based hierarchical node tree:

```
Root (NodeId)
  |-- MeshNode (transform, MeshId + MaterialId)
  |     |-- ChildMesh (inherits parent transform)
  |-- LightNode (Directional/Point/Spot)
  |-- EmptyNode (transform group)
        |-- Child1
        |-- Child2
```

- **Lazy transform propagation** -- world transforms recomputed only for dirty subtrees
- **Frustum culling** -- 6-plane frustum extracted from view-projection matrix; AABB test per node
- **LOD system** -- screen-space size threshold selects mesh detail level
- **Renderable enum** -- Mesh, Light (extensible for terrain, text, custom)

## Immediate Mode Design

Per-frame draw calls with no retained state:
- All calls push vertices into batched buffers (lines, triangles)
- Batched per primitive type to minimize draw calls
- Composable with scene graph -- both submit to the same RenderFrame
- Buffer cleared each frame

## Crate Dependency Graph

```
penumbra-backend (trait only, no GPU dep)
  |
  +-- penumbra-core (Renderer, RenderFrame, Material, DrawCall)
  |     |
  |     +-- penumbra-camera (perspective, ortho, controllers)
  |     +-- penumbra-scene (scene graph, depends on core + camera)
  |     +-- penumbra-pbr (PBR pipeline, lights, shaders)
  |     +-- penumbra-shadow (cascaded shadow maps)
  |     +-- penumbra-instance (GPU instanced rendering)
  |     +-- penumbra-terrain (tile streaming)
  |     +-- penumbra-atmosphere (sky, fog)
  |     +-- penumbra-post (post-processing)
  |     +-- penumbra-text (SDF text)
  |     +-- penumbra-immediate (immediate mode)
  |     +-- penumbra-asset (glTF, OBJ, image loading)
  |
  +-- penumbra-wgpu (default backend implementation)
  +-- penumbra-compute (compute shader abstraction)
  +-- penumbra-winit (winit integration)
  +-- penumbra-web (WASM target)

penumbra-geo (standalone, no rendering dependency)
```

## Geospatial Design

- WGS84 ellipsoid model with double-precision (DVec3)
- Coordinate systems: WGS84 (lat/lon/alt) <-> ECEF <-> ENU (local cartesian)
- Scene-local coordinates use ENU centered on a geographic origin
- Tile math: Web Mercator (EPSG:3857) for XYZ tile sources
- Terrain-RGB elevation decoding (Mapbox format)

## Performance Architecture

- **GPU frustum culling** -- compute shader tests each instance against camera frustum, produces indirect draw buffer
- **Instanced draw calls** -- single draw call per entity type regardless of count (2-4 calls for 27K entities)
- **Instance buffer upload** -- single write_buffer per batch per frame (27K x 96 bytes = 2.5MB)
- **SDF text batching** -- all visible labels rendered in 1-3 draw calls via glyph instancing
- **LRU tile cache** -- tiles stay on GPU until evicted; parent tile shown during async load
- **Deferred shading** -- single lighting pass regardless of light count
