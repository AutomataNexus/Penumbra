//! penumbra-wgpu — Default wgpu backend implementation for Penumbra.
//!
//! Implements [`RenderBackend`] using wgpu, supporting Vulkan, Metal, DX12, WebGPU, and WebGL2.

mod backend;
mod convert;
mod resources;

pub use backend::{WgpuBackend, WgpuConfig};
