//! penumbra-core — Core types, Renderer, and RenderFrame for the Penumbra 3D rendering SDK.

mod draw;
mod frame;
mod material;
mod renderer;

pub use draw::DrawCall;
pub use frame::{CameraUniforms, RenderFrame};
pub use material::{AlphaMode, Material, MaterialId};
pub use renderer::{FrameStats, Renderer, RendererConfig};

// Re-export math types from glam
pub use glam::{DVec3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

// Re-export backend types that consumers need
pub use penumbra_backend::{
    Aabb, BackendCapabilities, BackendError, BindGroupId, BufferDescriptor, BufferId, BufferUsage,
    GpuBuffer, GpuMesh, GpuTexture, MeshDescriptor, MeshId, PipelineId, RenderBackend, Rgb, Rgba,
    TextureDescriptor, TextureFormat, TextureId, TextureUsage, Vertex,
};
