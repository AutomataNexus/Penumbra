//! penumbra-compute -- Compute shader abstraction for Penumbra.

use penumbra_backend::{BindGroupId, ComputePipelineId};
use thiserror::Error;

// ── Errors ──

#[derive(Debug, Error)]
pub enum ComputeError {
    #[error("Pipeline not found")]
    PipelineNotFound,
    #[error("Invalid workgroup size: {0}")]
    InvalidWorkgroupSize(String),
    #[error("Backend error: {0}")]
    Backend(#[from] penumbra_backend::BackendError),
}

// ── Compute task ──

/// A single compute dispatch.
#[derive(Debug, Clone)]
pub struct ComputeTask {
    pub pipeline: ComputePipelineId,
    pub bind_groups: Vec<BindGroupId>,
    pub workgroups: [u32; 3],
}

// ── Compute scheduler ──

/// Queues and executes compute tasks.
pub struct ComputeScheduler {
    tasks: Vec<ComputeTask>,
}

impl ComputeScheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: ComputeTask) {
        self.tasks.push(task);
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Execute all queued tasks on the given backend.
    pub fn execute(&mut self, backend: &mut dyn penumbra_backend::RenderBackend) {
        let handle = backend.begin_compute_pass();
        for task in &self.tasks {
            backend.set_compute_pipeline(handle, task.pipeline);
            for (i, &bg) in task.bind_groups.iter().enumerate() {
                backend.set_compute_bind_group(handle, i as u32, bg);
            }
            backend.dispatch(handle, task.workgroups[0], task.workgroups[1], task.workgroups[2]);
        }
        backend.end_compute_pass(handle);
        self.tasks.clear();
    }
}

impl Default for ComputeScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ── GPU frustum culling ──

/// Configuration for GPU-based frustum culling.
#[derive(Debug, Clone)]
pub struct GpuCullingConfig {
    pub workgroup_size: u32,
}

impl Default for GpuCullingConfig {
    fn default() -> Self {
        Self { workgroup_size: 64 }
    }
}

/// GPU frustum culling dispatch helper.
pub struct GpuCulling {
    pub config: GpuCullingConfig,
}

impl GpuCulling {
    pub fn new(config: GpuCullingConfig) -> Self {
        Self { config }
    }

    /// Calculate the number of workgroups needed for the given instance count.
    pub fn workgroup_count(&self, instance_count: u32) -> u32 {
        instance_count.div_ceil(self.config.workgroup_size)
    }
}

// ── Shaders ──

pub const FRUSTUM_CULL_WGSL: &str = include_str!("shaders/frustum_cull.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_add_clear() {
        let mut sched = ComputeScheduler::new();
        assert_eq!(sched.task_count(), 0);

        sched.add_task(ComputeTask {
            pipeline: ComputePipelineId(1),
            bind_groups: vec![BindGroupId(0)],
            workgroups: [4, 1, 1],
        });
        sched.add_task(ComputeTask {
            pipeline: ComputePipelineId(2),
            bind_groups: vec![],
            workgroups: [8, 8, 1],
        });
        assert_eq!(sched.task_count(), 2);

        sched.clear();
        assert_eq!(sched.task_count(), 0);
    }

    #[test]
    fn workgroup_count_1000_div_64() {
        let culling = GpuCulling::new(GpuCullingConfig { workgroup_size: 64 });
        assert_eq!(culling.workgroup_count(1000), 16);
    }

    #[test]
    fn workgroup_count_exact() {
        let culling = GpuCulling::new(GpuCullingConfig { workgroup_size: 64 });
        assert_eq!(culling.workgroup_count(128), 2);
    }

    #[test]
    fn workgroup_count_remainder() {
        let culling = GpuCulling::new(GpuCullingConfig { workgroup_size: 64 });
        assert_eq!(culling.workgroup_count(65), 2);
    }
}
