//! Camera types: perspective and orthographic projections.

use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// A perspective camera defined by position, target, field-of-view, and clipping planes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectiveCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// Vertical field of view in **degrees**.
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl PerspectiveCamera {
    /// Compute the right-handed view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Compute the right-handed perspective projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y.to_radians(), self.aspect, self.near, self.far)
    }

    /// Convenience: `projection * view`.
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: 60.0,
            near: 0.1,
            far: 1000.0,
            aspect: 16.0 / 9.0,
        }
    }
}

/// An orthographic camera.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthographicCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicCamera {
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }

    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            left: -10.0,
            right: 10.0,
            bottom: -10.0,
            top: 10.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Unified camera enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Camera {
    Perspective(PerspectiveCamera),
    Orthographic(OrthographicCamera),
}

impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        match self {
            Camera::Perspective(c) => c.view_matrix(),
            Camera::Orthographic(c) => c.view_matrix(),
        }
    }

    pub fn projection_matrix(&self) -> Mat4 {
        match self {
            Camera::Perspective(c) => c.projection_matrix(),
            Camera::Orthographic(c) => c.projection_matrix(),
        }
    }

    pub fn view_projection(&self) -> Mat4 {
        match self {
            Camera::Perspective(c) => c.view_projection(),
            Camera::Orthographic(c) => c.view_projection(),
        }
    }
}
