//! Fly controller — WASD + mouse-look first-person camera.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::camera::PerspectiveCamera;

/// A first-person fly controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlyController {
    pub position: Vec3,
    /// Horizontal rotation in radians.
    pub yaw: f32,
    /// Vertical rotation in radians (clamped to ±89 degrees).
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub sprint_mult: f32,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl Default for FlyController {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
            sensitivity: 0.003,
            sprint_mult: 2.0,
            fov_y: 60.0,
            near: 0.1,
            far: 1000.0,
            aspect: 16.0 / 9.0,
        }
    }
}

impl FlyController {
    /// Forward direction vector derived from yaw and pitch.
    fn forward(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
    }

    /// Right direction vector (perpendicular to forward on the XZ plane).
    fn right(&self) -> Vec3 {
        // right = forward x up, but we want a horizontal-only right vector
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()).normalize()
    }

    /// Update yaw/pitch from mouse delta (pixels).
    pub fn handle_mouse_move(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * self.sensitivity;
        self.pitch = (self.pitch - dy * self.sensitivity)
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
    }

    pub fn move_forward(&mut self, dt: f32) {
        self.position += self.forward() * self.speed * dt;
    }

    pub fn move_back(&mut self, dt: f32) {
        self.position -= self.forward() * self.speed * dt;
    }

    pub fn move_left(&mut self, dt: f32) {
        self.position -= self.right() * self.speed * dt;
    }

    pub fn move_right(&mut self, dt: f32) {
        self.position += self.right() * self.speed * dt;
    }

    /// Build a [`PerspectiveCamera`] from the current fly state.
    pub fn camera(&self) -> PerspectiveCamera {
        let fwd = self.forward();
        PerspectiveCamera {
            position: self.position,
            target: self.position + fwd,
            up: Vec3::Y,
            fov_y: self.fov_y,
            near: self.near,
            far: self.far,
            aspect: self.aspect,
        }
    }
}
