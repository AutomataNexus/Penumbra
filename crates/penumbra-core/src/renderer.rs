use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

use penumbra_backend::{
    BackendCapabilities, BackendError, BufferDescriptor, GpuBuffer, GpuMesh, GpuTexture,
    MeshDescriptor, MeshId, RenderBackend, TextureDescriptor, TextureId, TextureFormat,
};

use crate::frame::RenderFrame;
use crate::material::{Material, MaterialId};

/// Renderer configuration.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub msaa_samples: u32,
    pub hdr: bool,
    pub vsync: bool,
    pub max_instances: u32,
    pub tile_cache_mb: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            msaa_samples: 1,
            hdr: false,
            vsync: true,
            max_instances: 65536,
            tile_cache_mb: 256,
        }
    }
}

/// Per-frame performance statistics.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    pub frame_time_ms: f32,
    pub fps: f32,
    pub draw_calls: u32,
    pub triangles: u64,
    pub instances: u32,
    pub tiles_loaded: u32,
    pub tiles_streaming: u32,
    pub gpu_memory_mb: u32,
}

/// The central renderer. Owns the backend and manages frame lifecycle.
pub struct Renderer {
    backend: Box<dyn RenderBackend>,
    config: RendererConfig,
    frame_stats: FrameStats,
    start_time: Instant,
    last_frame_time: Instant,
    frame_count: u64,
    materials: HashMap<MaterialId, Material>,
    next_material_id: u64,
}

impl Renderer {
    pub fn new(backend: impl RenderBackend + 'static, config: RendererConfig) -> Self {
        let now = Instant::now();
        info!(
            backend = backend.backend_name(),
            width = config.width,
            height = config.height,
            "Renderer created"
        );
        Self {
            backend: Box::new(backend),
            config,
            frame_stats: FrameStats::default(),
            start_time: now,
            last_frame_time: now,
            frame_count: 0,
            materials: HashMap::new(),
            next_material_id: 1,
        }
    }

    pub fn begin_frame(&mut self) -> Result<RenderFrame, BackendError> {
        let now = Instant::now();
        let time = now.duration_since(self.start_time).as_secs_f64();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();

        self.backend.begin_frame()?;

        Ok(RenderFrame::new(self.config.width, self.config.height, time, delta))
    }

    pub fn end_frame(&mut self, frame: RenderFrame) -> Result<(), BackendError> {
        let now = Instant::now();
        let frame_time_ms = now
            .duration_since(self.last_frame_time)
            .as_secs_f32()
            * 1000.0;

        self.frame_stats.frame_time_ms = frame_time_ms;
        self.frame_stats.fps = if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        };
        self.frame_stats.draw_calls = frame.draw_count();
        self.last_frame_time = now;
        self.frame_count += 1;

        self.backend.end_frame()?;
        self.backend.present()?;
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.backend.resize(width, height);
        info!(width, height, "Renderer resized");
    }

    pub fn stats(&self) -> &FrameStats {
        &self.frame_stats
    }

    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn capabilities(&self) -> BackendCapabilities {
        self.backend.capabilities()
    }

    pub fn backend_name(&self) -> &str {
        self.backend.backend_name()
    }

    // ── Resource creation (delegate to backend) ──

    pub fn create_mesh(&mut self, desc: MeshDescriptor) -> Result<GpuMesh, BackendError> {
        self.backend.create_mesh(desc)
    }

    pub fn create_texture(&mut self, desc: TextureDescriptor) -> Result<GpuTexture, BackendError> {
        self.backend.create_texture(desc)
    }

    pub fn create_buffer(&mut self, desc: BufferDescriptor) -> Result<GpuBuffer, BackendError> {
        self.backend.create_buffer(desc)
    }

    pub fn destroy_mesh(&mut self, id: MeshId) {
        self.backend.destroy_mesh(id);
    }

    pub fn destroy_texture(&mut self, id: TextureId) {
        self.backend.destroy_texture(id);
    }

    // ── Material management ──

    pub fn add_material(&mut self, mut material: Material) -> MaterialId {
        let id = MaterialId(self.next_material_id);
        self.next_material_id += 1;
        material.id = id;
        self.materials.insert(id, material);
        id
    }

    pub fn get_material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(&id)
    }

    pub fn get_material_mut(&mut self, id: MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(&id)
    }

    pub fn remove_material(&mut self, id: MaterialId) -> Option<Material> {
        self.materials.remove(&id)
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.backend.surface_format()
    }

    pub fn backend_mut(&mut self) -> &mut dyn RenderBackend {
        &mut *self.backend
    }

    pub fn backend(&self) -> &dyn RenderBackend {
        &*self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_config_default() {
        let config = RendererConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert_eq!(config.msaa_samples, 1);
        assert!(!config.hdr);
        assert!(config.vsync);
        assert_eq!(config.max_instances, 65536);
    }

    #[test]
    fn frame_stats_default() {
        let stats = FrameStats::default();
        assert_eq!(stats.fps, 0.0);
        assert_eq!(stats.draw_calls, 0);
    }
}
