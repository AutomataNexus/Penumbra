use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::draw::DrawCall;

/// Uniform data for the camera, uploaded to GPU each frame.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniforms {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub inverse_view: [[f32; 4]; 4],
    pub inverse_projection: [[f32; 4]; 4],
    pub camera_position: [f32; 3],
    pub _pad0: f32,
    pub near: f32,
    pub far: f32,
    pub _pad1: [f32; 2],
}

impl CameraUniforms {
    pub fn from_matrices(view: Mat4, projection: Mat4, near: f32, far: f32) -> Self {
        let view_projection = projection * view;
        let inverse_view = view.inverse();
        let camera_position = inverse_view.col(3).truncate();
        Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            view_projection: view_projection.to_cols_array_2d(),
            inverse_view: inverse_view.to_cols_array_2d(),
            inverse_projection: projection.inverse().to_cols_array_2d(),
            camera_position: camera_position.to_array(),
            _pad0: 0.0,
            near,
            far,
            _pad1: [0.0; 2],
        }
    }
}

/// A single frame being rendered. All draw calls are collected here
/// and submitted to the backend when the frame ends.
pub struct RenderFrame {
    pub width: u32,
    pub height: u32,
    pub time: f64,
    pub delta: f32,
    pub camera: CameraUniforms,
    draws: Vec<DrawCall>,
}

impl RenderFrame {
    pub(crate) fn new(width: u32, height: u32, time: f64, delta: f32) -> Self {
        Self {
            width,
            height,
            time,
            delta,
            camera: CameraUniforms::from_matrices(
                Mat4::IDENTITY,
                Mat4::IDENTITY,
                0.1,
                1000.0,
            ),
            draws: Vec::with_capacity(1024),
        }
    }

    pub fn submit(&mut self, draw: DrawCall) {
        self.draws.push(draw);
    }

    pub fn submit_batch(&mut self, draws: impl IntoIterator<Item = DrawCall>) {
        self.draws.extend(draws);
    }

    pub fn draw_calls(&self) -> &[DrawCall] {
        &self.draws
    }

    pub fn draw_count(&self) -> u32 {
        self.draws.len() as u32
    }

    pub fn set_camera(&mut self, view: Mat4, projection: Mat4, near: f32, far: f32) {
        self.camera = CameraUniforms::from_matrices(view, projection, near, far);
    }

    pub fn sort_draws(&mut self) {
        self.draws.sort_unstable_by_key(|d| d.sort_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use penumbra_backend::{MeshId, PipelineId};
    use crate::material::MaterialId;

    #[test]
    fn frame_collects_draws() {
        let mut frame = RenderFrame::new(1920, 1080, 0.0, 0.016);
        assert_eq!(frame.draw_count(), 0);

        frame.submit(DrawCall::new(
            MeshId(1),
            MaterialId(1),
            PipelineId(1),
            Mat4::IDENTITY,
        ));
        assert_eq!(frame.draw_count(), 1);
    }

    #[test]
    fn camera_uniforms_position() {
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 5.0, 10.0),
            Vec3::ZERO,
            Vec3::Y,
        );
        let proj = Mat4::perspective_rh(60_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
        let uniforms = CameraUniforms::from_matrices(view, proj, 0.1, 1000.0);
        let pos = Vec3::from_array(uniforms.camera_position);
        assert!((pos.x - 0.0).abs() < 0.001);
        assert!((pos.y - 5.0).abs() < 0.001);
        assert!((pos.z - 10.0).abs() < 0.001);
    }
}
