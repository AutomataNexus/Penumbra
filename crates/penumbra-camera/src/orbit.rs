//! Orbit controller — rotate around a target point with mouse + scroll.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::camera::PerspectiveCamera;

/// An orbit controller that revolves around a target point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitController {
    pub target: Vec3,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub sensitivity: f32,
    pub invert_y: bool,
    /// Vertical angle (polar) in radians. 0 = top, PI = bottom.
    pub phi: f32,
    /// Horizontal angle (azimuthal) in radians.
    pub theta: f32,
    /// Vertical FOV in degrees for the generated camera.
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl Default for OrbitController {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 5.0,
            min_distance: 0.5,
            max_distance: 100.0,
            sensitivity: 0.005,
            invert_y: false,
            phi: std::f32::consts::FRAC_PI_4,
            theta: 0.0,
            fov_y: 60.0,
            near: 0.1,
            far: 1000.0,
            aspect: 16.0 / 9.0,
        }
    }
}

impl OrbitController {
    /// Update angles from mouse delta (pixels).
    pub fn handle_mouse_move(&mut self, dx: f32, dy: f32) {
        self.theta += dx * self.sensitivity;
        let dy = if self.invert_y { -dy } else { dy };
        self.phi = (self.phi - dy * self.sensitivity).clamp(0.01, std::f32::consts::PI - 0.01);
    }

    /// Zoom in/out from scroll delta.
    pub fn handle_scroll(&mut self, delta: f32) {
        self.distance = (self.distance - delta).clamp(self.min_distance, self.max_distance);
    }

    /// Build a [`PerspectiveCamera`] from the current orbit state.
    pub fn camera(&self) -> PerspectiveCamera {
        let x = self.distance * self.phi.sin() * self.theta.sin();
        let y = self.distance * self.phi.cos();
        let z = self.distance * self.phi.sin() * self.theta.cos();

        PerspectiveCamera {
            position: self.target + Vec3::new(x, y, z),
            target: self.target,
            up: Vec3::Y,
            fov_y: self.fov_y,
            near: self.near,
            far: self.far,
            aspect: self.aspect,
        }
    }
}
