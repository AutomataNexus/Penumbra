//! penumbra-core — Core types, Renderer, and RenderFrame for the Penumbra 3D rendering SDK.

mod renderer;
mod frame;
mod material;
mod draw;

pub use renderer::{Renderer, RendererConfig, FrameStats};
pub use frame::{RenderFrame, CameraUniforms};
pub use material::{Material, MaterialId, AlphaMode};
pub use draw::DrawCall;

// Re-export math types from glam
pub use glam::{
    DVec3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4,
};

// Re-export backend types that consumers need
pub use penumbra_backend::{
    Aabb, BackendError, GpuBuffer, GpuMesh, GpuTexture, MeshId, TextureId, BufferId,
    PipelineId, BindGroupId, Rgba, Rgb, RenderBackend, BackendCapabilities,
    MeshDescriptor, TextureDescriptor, BufferDescriptor, TextureFormat, TextureUsage,
    BufferUsage, Vertex,
};
