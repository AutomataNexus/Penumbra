use std::ops::Range;

use crate::error::BackendError;
use crate::types::*;

// On native, RenderBackend must be Send + Sync for multi-threaded use.
// On WASM (single-threaded), wgpu types aren't Send/Sync, so we relax this.
#[cfg(not(target_arch = "wasm32"))]
mod thread_safety {
    pub trait MaybeSend: Send {}
    pub trait MaybeSync: Sync {}
    impl<T: Send> MaybeSend for T {}
    impl<T: Sync> MaybeSync for T {}
}
#[cfg(target_arch = "wasm32")]
mod thread_safety {
    pub trait MaybeSend {}
    pub trait MaybeSync {}
    impl<T> MaybeSend for T {}
    impl<T> MaybeSync for T {}
}
pub use thread_safety::{MaybeSend, MaybeSync};

/// The central GPU abstraction trait. All Penumbra feature crates depend only on this trait,
/// never on a specific GPU API. Implement this trait to plug in a custom backend.
pub trait RenderBackend: MaybeSend + MaybeSync {
    // ── Resource creation ──

    fn create_mesh(&mut self, desc: MeshDescriptor) -> Result<GpuMesh, BackendError>;
    fn create_texture(&mut self, desc: TextureDescriptor) -> Result<GpuTexture, BackendError>;
    fn create_buffer(&mut self, desc: BufferDescriptor) -> Result<GpuBuffer, BackendError>;
    fn create_pipeline(&mut self, desc: PipelineDescriptor) -> Result<PipelineId, BackendError>;
    fn create_compute_pipeline(
        &mut self,
        desc: ComputePipelineDescriptor,
    ) -> Result<ComputePipelineId, BackendError>;
    fn create_bind_group(
        &mut self,
        layout: &BindGroupLayoutDescriptor,
        entries: &[BindGroupEntry],
    ) -> Result<BindGroupId, BackendError>;

    // ── Resource destruction ──

    fn destroy_mesh(&mut self, id: MeshId);
    fn destroy_texture(&mut self, id: TextureId);
    fn destroy_buffer(&mut self, id: BufferId);

    // ── Buffer updates ──

    fn write_buffer(&mut self, id: BufferId, offset: u64, data: &[u8]);
    fn read_buffer(&mut self, id: BufferId, offset: u64, len: u64) -> Vec<u8>;

    // ── Texture updates ──

    fn write_texture(&mut self, id: TextureId, region: TextureRegion, data: &[u8]);

    // ── Frame lifecycle ──

    fn begin_frame(&mut self) -> Result<(), BackendError>;
    fn end_frame(&mut self) -> Result<(), BackendError>;
    fn present(&mut self) -> Result<(), BackendError>;

    // ── Render pass ──

    fn begin_render_pass(&mut self, desc: RenderPassDescriptor) -> RenderPassHandle;
    fn end_render_pass(&mut self, handle: RenderPassHandle);
    fn set_pipeline(&mut self, handle: RenderPassHandle, pipeline: PipelineId);
    fn set_bind_group(&mut self, handle: RenderPassHandle, index: u32, group: BindGroupId);
    fn set_vertex_buffer(&mut self, handle: RenderPassHandle, slot: u32, buffer: BufferSlice);
    fn set_index_buffer(&mut self, handle: RenderPassHandle, buffer: BufferSlice);
    fn draw(&mut self, handle: RenderPassHandle, vertices: Range<u32>, instances: Range<u32>);
    fn draw_indexed(
        &mut self,
        handle: RenderPassHandle,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    );

    // ── Compute pass ──

    fn begin_compute_pass(&mut self) -> ComputePassHandle;
    fn end_compute_pass(&mut self, handle: ComputePassHandle);
    fn set_compute_pipeline(&mut self, handle: ComputePassHandle, pipeline: ComputePipelineId);
    fn set_compute_bind_group(&mut self, handle: ComputePassHandle, index: u32, group: BindGroupId);
    fn dispatch(&mut self, handle: ComputePassHandle, x: u32, y: u32, z: u32);

    // ── Capabilities ──

    fn capabilities(&self) -> BackendCapabilities;
    fn backend_name(&self) -> &str;

    // ── Viewport / surface ──

    fn resize(&mut self, width: u32, height: u32);
    fn surface_format(&self) -> TextureFormat;
    fn surface_texture(&self) -> Result<TextureId, BackendError>;
}

/// An entry for a bind group — refers to a bound resource.
#[derive(Debug, Clone)]
pub enum BindGroupEntry {
    Buffer {
        binding: u32,
        buffer: BufferId,
        offset: u64,
        size: u64,
    },
    Texture {
        binding: u32,
        texture: TextureId,
    },
    Sampler {
        binding: u32,
    },
}
