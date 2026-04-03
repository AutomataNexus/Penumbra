# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- Initial project scaffolding: 18 sub-crates + root umbrella crate
- `penumbra-backend`: RenderBackend trait with MaybeSend/MaybeSync for WASM compat
- `penumbra-core`: Renderer (Result-returning frame lifecycle), RenderFrame, Material (HashMap storage), DrawCall
- `penumbra-wgpu`: Full wgpu backend -- headless, resource creation, render/compute pass replay, bind groups, real buffer readback, per-format bpp handling
- `penumbra-geo`: WGS84 geodesy (ECEF, ENU, haversine, bearing, great circle, tile math with pole clamping)
- `penumbra-camera`: Perspective, orthographic, orbit, fly, globe controllers; ray casting with NaN-safe AABB
- `penumbra-scene`: SlotMap scene graph, transform hierarchy, frustum culling, LOD
- `penumbra-pbr`: PBR pipeline, Cook-Torrance BRDF shaders, lights with shadow flag in uniforms
- `penumbra-asset`: glTF 2.0, OBJ, image loading, cube/sphere/plane generators
- `penumbra-instance`: 27K+ entity instancing, CPU frustum culling, batch manager
- `penumbra-shadow`: Cascaded shadow maps (logarithmic splits), point light cubemaps, PCF shader
- `penumbra-post`: PostPipeline builder, tone mapping (ACES/Reinhard/Uncharted2), bloom, SSAO, FXAA, color grading, vignette, chromatic aberration, sharpen
- `penumbra-immediate`: Line/shape/billboard/grid/filled rect drawing with batching
- `penumbra-compute`: Compute task scheduler, GPU culling abstraction, frustum cull shader
- `penumbra-text`: SDF font atlas, text layout, billboard/2D text, glyph batching, SDF shader
- `penumbra-terrain`: Tile streaming (XYZ sources), LRU cache, terrain mesh gen, Terrain-RGB decode
- `penumbra-atmosphere`: Bruneton-Neyret config, Rayleigh/Mie scattering, fog, atmosphere shader
- `penumbra-winit`: Full winit event loop (ApplicationHandler), keyboard/mouse/scroll input, PenumbraApp trait
- `penumbra-web`: Browser surface, fetch tile loading, requestAnimationFrame loop, real wasm-bindgen/web-sys impl
- `penumbra` (root): Re-exports all sub-crates, criterion benchmarks
- 8 WGSL shaders (PBR vertex, Cook-Torrance lighting, shadow PCF, atmosphere, SDF text, tone mapping, FXAA, frustum cull)
- 6 examples: hello_cube, pbr_scene, instanced_entities, globe, tactical_globe, wasm
- 3 criterion benchmarks: instancing, tile_streaming, full_scene
- 116 unit tests across all crates
- WASM compilation verified (wasm32-unknown-unknown), wasm-pack binary built
- Project docs: README, ARCHITECTURE, PROGRESS, PERFORMANCE, CHANGELOG, LICENSE
