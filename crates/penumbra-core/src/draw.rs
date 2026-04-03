use crate::material::MaterialId;
use glam::Mat4;
use penumbra_backend::{BindGroupId, MeshId, PipelineId};

/// A single draw call to be submitted to the GPU.
#[derive(Debug, Clone)]
pub struct DrawCall {
    pub mesh: MeshId,
    pub material: MaterialId,
    pub pipeline: PipelineId,
    pub bind_groups: Vec<BindGroupId>,
    pub transform: Mat4,
    pub instance_count: u32,
    pub sort_key: u64,
}

impl DrawCall {
    pub fn new(mesh: MeshId, material: MaterialId, pipeline: PipelineId, transform: Mat4) -> Self {
        Self {
            mesh,
            material,
            pipeline,
            bind_groups: Vec::new(),
            transform,
            instance_count: 1,
            sort_key: 0,
        }
    }
}
