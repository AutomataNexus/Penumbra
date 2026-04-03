//! # Penumbra
//!
//! General-purpose 3D rendering SDK for Rust.
//!
//! Penumbra sits between raw GPU bindings (wgpu) and full game engines (Bevy).
//! It provides a complete 3D rendering toolkit: PBR, instanced rendering,
//! terrain streaming, atmospheric scattering, shadows, post-processing,
//! text, and compute shaders — all running on desktop (Vulkan/Metal/DX12)
//! and browser (WebGPU/WebGL2 via WASM).
//!
//! ## Crate Re-exports
//!
//! This top-level crate re-exports all Penumbra sub-crates for convenience.
//! You can depend on `penumbra` alone and access everything, or depend on
//! individual crates for finer-grained control.

pub use penumbra_core as core;
pub use penumbra_backend as backend;
pub use penumbra_wgpu as wgpu_backend;
pub use penumbra_scene as scene;
pub use penumbra_pbr as pbr;
pub use penumbra_instance as instance;
pub use penumbra_terrain as terrain;
pub use penumbra_atmosphere as atmosphere;
pub use penumbra_post as post;
pub use penumbra_shadow as shadow;
pub use penumbra_text as text;
pub use penumbra_compute as compute;
pub use penumbra_geo as geo;
pub use penumbra_immediate as immediate;
pub use penumbra_camera as camera;
pub use penumbra_asset as asset;
pub use penumbra_winit as winit_integration;
pub use penumbra_web as web;
