//! # penumbra-camera
//!
//! Camera system for the Penumbra 3D rendering SDK.
//!
//! Provides perspective and orthographic cameras, orbit and fly controllers,
//! and ray-casting utilities (screen-to-ray, plane / AABB intersection).

pub mod camera;
pub mod fly;
pub mod globe;
pub mod orbit;
pub mod ray;

// Re-exports for convenience.
pub use camera::{Camera, OrthographicCamera, PerspectiveCamera};
pub use fly::FlyController;
pub use globe::GlobeController;
pub use orbit::OrbitController;
pub use ray::{screen_to_ray, Ray};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn perspective_view_matrix_look_at() {
        let cam = PerspectiveCamera {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: 60.0,
            near: 0.1,
            far: 100.0,
            aspect: 1.0,
        };
        let view = cam.view_matrix();
        // Transforming the camera position by the view matrix should yield the origin.
        let transformed = view.transform_point3(cam.position);
        assert!(transformed.length() < 1e-5, "camera position should map to origin in view space");
    }

    #[test]
    fn orthographic_projection_is_finite() {
        let cam = OrthographicCamera::default();
        let proj = cam.projection_matrix();
        for col in 0..4 {
            for row in 0..4 {
                assert!(proj.col(col)[row].is_finite());
            }
        }
    }

    #[test]
    fn camera_enum_delegates() {
        let pcam = PerspectiveCamera::default();
        let expected_vp = pcam.view_projection();
        let cam = Camera::Perspective(pcam);
        let vp = cam.view_projection();
        assert!((vp - expected_vp).abs_diff_eq(glam::Mat4::ZERO, 1e-6));
    }

    #[test]
    fn orbit_controller_camera_position() {
        use std::f32::consts::FRAC_PI_2;
        let ctrl = OrbitController {
            target: Vec3::ZERO,
            distance: 10.0,
            phi: FRAC_PI_2,
            theta: 0.0,
            ..Default::default()
        };
        let cam = ctrl.camera();
        // phi=PI/2, theta=0 => position = (0, 0, 10)
        assert!((cam.position - Vec3::new(0.0, 0.0, 10.0)).length() < 1e-4);
    }

    #[test]
    fn orbit_scroll_clamps() {
        let mut ctrl = OrbitController {
            distance: 5.0,
            min_distance: 1.0,
            max_distance: 20.0,
            ..Default::default()
        };
        ctrl.handle_scroll(100.0); // large zoom in
        assert!(ctrl.distance >= ctrl.min_distance);
        ctrl.handle_scroll(-200.0); // large zoom out
        assert!(ctrl.distance <= ctrl.max_distance);
    }

    #[test]
    fn fly_controller_forward_back() {
        let mut ctrl = FlyController {
            position: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            speed: 10.0,
            ..Default::default()
        };
        ctrl.move_forward(1.0);
        // yaw=0, pitch=0 => forward = (0, 0, 1)
        assert!(ctrl.position.z > 0.0, "should move along +Z when yaw=0");
        ctrl.move_back(1.0);
        assert!(ctrl.position.z.abs() < 1e-5, "should return to origin");
    }
}
