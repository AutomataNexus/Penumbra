//! penumbra-instance -- Instanced rendering for Penumbra.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};
use penumbra_backend::MeshId;
use thiserror::Error;

// ── Errors ──

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("Batch not found: {0:?}")]
    BatchNotFound(InstanceBatchId),
    #[error("Capacity exceeded: max {max}, requested {requested}")]
    CapacityExceeded { max: usize, requested: usize },
    #[error("Invalid instance data")]
    InvalidData,
}

// ── Instance data (96 bytes) ──

/// Per-instance GPU data. 96 bytes, suitable for vertex buffer upload.
///
/// Layout:
///   transform: mat4 (64 bytes)
///   color: vec4 (16 bytes)
///   uv_offset: vec2 (8 bytes)
///   uv_scale: vec2 (8 bytes -- but we pack user_data into the remaining space)
///
/// Total = 64 + 16 + 8 + 4 + 4 = 96 bytes
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub transform: [f32; 16],  // 64 bytes (mat4)
    pub color: [f32; 4],       // 16 bytes
    pub uv_offset: [f32; 2],   // 8 bytes
    pub uv_scale: [f32; 2],    // 8 bytes
}

// ── Batch ID ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceBatchId(pub u64);

// ── Batch descriptor ──

#[derive(Debug, Clone)]
pub struct InstanceBatchDesc {
    pub mesh: MeshId,
    pub max_instances: usize,
    pub label: Option<String>,
}

// ── Batch ──

#[derive(Debug, Clone)]
pub struct InstanceBatch {
    pub id: InstanceBatchId,
    pub mesh: MeshId,
    pub instances: Vec<InstanceData>,
    pub max_instances: usize,
}

impl InstanceBatch {
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

// ── Manager ──

pub struct InstanceManager {
    batches: Vec<InstanceBatch>,
    next_id: u64,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
            next_id: 0,
        }
    }

    pub fn create_batch(&mut self, desc: InstanceBatchDesc) -> InstanceBatchId {
        let id = InstanceBatchId(self.next_id);
        self.next_id += 1;
        self.batches.push(InstanceBatch {
            id,
            mesh: desc.mesh,
            instances: Vec::new(),
            max_instances: desc.max_instances,
        });
        id
    }

    pub fn update_batch(
        &mut self,
        id: InstanceBatchId,
        instances: Vec<InstanceData>,
    ) -> Result<(), InstanceError> {
        let batch = self
            .batches
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(InstanceError::BatchNotFound(id))?;
        if instances.len() > batch.max_instances {
            return Err(InstanceError::CapacityExceeded {
                max: batch.max_instances,
                requested: instances.len(),
            });
        }
        batch.instances = instances;
        Ok(())
    }

    pub fn remove_batch(&mut self, id: InstanceBatchId) -> Result<(), InstanceError> {
        let pos = self
            .batches
            .iter()
            .position(|b| b.id == id)
            .ok_or(InstanceError::BatchNotFound(id))?;
        self.batches.remove(pos);
        Ok(())
    }

    pub fn get_batch(&self, id: InstanceBatchId) -> Option<&InstanceBatch> {
        self.batches.iter().find(|b| b.id == id)
    }

    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── CPU frustum culling fallback ──

/// CPU-side frustum culling. Returns indices of visible instances.
pub fn cpu_frustum_cull(instances: &[InstanceData], view_projection: Mat4) -> Vec<u32> {
    let mut visible = Vec::new();
    for (i, inst) in instances.iter().enumerate() {
        // Extract translation from the transform matrix (column 3)
        let pos = Vec4::new(inst.transform[12], inst.transform[13], inst.transform[14], 1.0);
        let clip = view_projection * pos;

        // Simple point-in-frustum check (w-clip test)
        let w = clip.w.abs().max(0.001);
        if clip.x >= -w && clip.x <= w && clip.y >= -w && clip.y <= w && clip.z >= 0.0 && clip.z <= w
        {
            visible.push(i as u32);
        }
    }
    visible
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_data_size_96() {
        assert_eq!(std::mem::size_of::<InstanceData>(), 96);
    }

    #[test]
    fn batch_update_query() {
        let mut mgr = InstanceManager::new();
        let id = mgr.create_batch(InstanceBatchDesc {
            mesh: MeshId(0),
            max_instances: 100,
            label: None,
        });
        let data = vec![InstanceData::zeroed(); 10];
        mgr.update_batch(id, data).unwrap();
        let batch = mgr.get_batch(id).unwrap();
        assert_eq!(batch.instance_count(), 10);
    }

    #[test]
    fn capacity_exceeded() {
        let mut mgr = InstanceManager::new();
        let id = mgr.create_batch(InstanceBatchDesc {
            mesh: MeshId(0),
            max_instances: 5,
            label: None,
        });
        let data = vec![InstanceData::zeroed(); 10];
        let result = mgr.update_batch(id, data);
        assert!(result.is_err());
        match result.unwrap_err() {
            InstanceError::CapacityExceeded { max, requested } => {
                assert_eq!(max, 5);
                assert_eq!(requested, 10);
            }
            _ => panic!("Expected CapacityExceeded"),
        }
    }

    #[test]
    fn cpu_culling() {
        // Use identity VP => only points inside NDC cube are visible
        let vp = Mat4::IDENTITY;
        let mut instances = Vec::new();

        // Instance at origin -- should be visible
        let mut d = InstanceData::zeroed();
        d.transform[0] = 1.0;
        d.transform[5] = 1.0;
        d.transform[10] = 1.0;
        d.transform[15] = 1.0;
        // position at (0,0,0.5) -- inside clip
        d.transform[14] = 0.5;
        instances.push(d);

        // Instance far away -- outside clip
        let mut d2 = InstanceData::zeroed();
        d2.transform[0] = 1.0;
        d2.transform[5] = 1.0;
        d2.transform[10] = 1.0;
        d2.transform[15] = 1.0;
        d2.transform[12] = 100.0;
        instances.push(d2);

        let visible = cpu_frustum_cull(&instances, vp);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0], 0);
    }

    #[test]
    fn remove_batch() {
        let mut mgr = InstanceManager::new();
        let id = mgr.create_batch(InstanceBatchDesc {
            mesh: MeshId(0),
            max_instances: 10,
            label: None,
        });
        assert_eq!(mgr.batch_count(), 1);
        mgr.remove_batch(id).unwrap();
        assert_eq!(mgr.batch_count(), 0);
    }
}
