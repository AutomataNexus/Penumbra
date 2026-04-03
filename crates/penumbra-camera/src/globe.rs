//! Globe controller — orbit around WGS84 globe (altitude-based zoom, tilt).

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::camera::PerspectiveCamera;

/// A camera controller for orbiting a WGS84 globe.
///
/// Altitude is in meters above the ground. Latitude and longitude are in degrees.
/// The controller converts geographic position to a 3D camera position internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobeController {
    /// Geographic latitude in degrees.
    pub latitude: f64,
    /// Geographic longitude in degrees.
    pub longitude: f64,
    /// Altitude above ground in meters.
    pub altitude: f64,
    /// Minimum altitude in meters.
    pub min_altitude: f64,
    /// Maximum altitude in meters (orbit view).
    pub max_altitude: f64,
    /// Camera tilt in degrees (0 = looking straight down, 90 = looking at horizon).
    pub tilt: f64,
    /// Minimum tilt angle in degrees.
    pub min_tilt: f64,
    /// Maximum tilt angle in degrees.
    pub max_tilt: f64,
    /// Heading in degrees (0 = north, 90 = east).
    pub heading: f64,
    /// Mouse/touch sensitivity.
    pub sensitivity: f64,
    /// Vertical FOV in degrees.
    pub fov_y: f32,
    /// Near clipping plane.
    pub near: f32,
    /// Far clipping plane.
    pub far: f32,
    /// Aspect ratio.
    pub aspect: f32,
}

impl Default for GlobeController {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: 10_000_000.0,
            min_altitude: 100.0,
            max_altitude: 20_000_000.0,
            tilt: 0.0,
            min_tilt: 0.0,
            max_tilt: 89.0,
            heading: 0.0,
            sensitivity: 0.005,
            fov_y: 60.0,
            near: 1.0,
            far: 100_000_000.0,
            aspect: 16.0 / 9.0,
        }
    }
}

impl GlobeController {
    /// Pan the camera by dragging (changes lat/lon).
    pub fn handle_mouse_move(&mut self, dx: f32, dy: f32) {
        // Scale pan sensitivity by altitude (higher = faster pan)
        let scale = self.altitude * self.sensitivity;
        self.longitude += dx as f64 * scale * 0.0001;
        self.latitude -= dy as f64 * scale * 0.0001;
        self.latitude = self.latitude.clamp(-85.0, 85.0);
        // Wrap longitude
        if self.longitude > 180.0 {
            self.longitude -= 360.0;
        }
        if self.longitude < -180.0 {
            self.longitude += 360.0;
        }
    }

    /// Zoom by changing altitude.
    pub fn handle_scroll(&mut self, delta: f32) {
        let factor = 1.0 - delta as f64 * 0.1;
        self.altitude = (self.altitude * factor).clamp(self.min_altitude, self.max_altitude);
    }

    /// Tilt the camera (change viewing angle).
    pub fn handle_tilt(&mut self, delta: f32) {
        self.tilt = (self.tilt + delta as f64).clamp(self.min_tilt, self.max_tilt);
    }

    /// Rotate the camera heading.
    pub fn handle_rotate(&mut self, delta: f32) {
        self.heading = (self.heading + delta as f64) % 360.0;
    }

    /// Build a PerspectiveCamera from the current globe state.
    ///
    /// Uses a simplified model where the globe center is at the origin.
    /// The camera looks at a point on the globe surface at (lat, lon) from
    /// the specified altitude and tilt angle.
    pub fn camera(&self) -> PerspectiveCamera {
        let lat_rad = (self.latitude as f32).to_radians();
        let lon_rad = (self.longitude as f32).to_radians();
        let heading_rad = (self.heading as f32).to_radians();
        let tilt_rad = (self.tilt as f32).to_radians();

        // Approximate Earth radius
        let earth_radius = 6_371_000.0_f32;

        // Surface point in ECEF-like coordinates
        let surface = Vec3::new(
            earth_radius * lat_rad.cos() * lon_rad.cos(),
            earth_radius * lat_rad.sin(),
            earth_radius * lat_rad.cos() * lon_rad.sin(),
        );

        // Up vector at the surface point (radial direction)
        let up_dir = surface.normalize();

        // Camera position: above the surface point, along the radial direction
        let alt = self.altitude as f32;
        let camera_distance = earth_radius + alt;

        // With tilt, the camera is offset from directly above
        let camera_pos = if tilt_rad.abs() < 0.001 {
            // Looking straight down
            surface.normalize() * camera_distance
        } else {
            // Offset camera backward based on tilt
            let forward = Vec3::new(
                -lon_rad.sin() * heading_rad.cos()
                    + lat_rad.sin() * lon_rad.cos() * heading_rad.sin(),
                -lat_rad.cos() * heading_rad.sin(),
                lon_rad.cos() * heading_rad.cos()
                    + lat_rad.sin() * lon_rad.sin() * heading_rad.sin(),
            )
            .normalize();
            let offset = forward * tilt_rad.sin() * alt;
            surface.normalize() * camera_distance + offset
        };

        let target = surface;

        PerspectiveCamera {
            position: camera_pos,
            target,
            up: up_dir,
            fov_y: self.fov_y,
            near: self.near,
            far: self.far,
            aspect: self.aspect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_globe_controller() {
        let ctrl = GlobeController::default();
        assert_eq!(ctrl.latitude, 0.0);
        assert_eq!(ctrl.longitude, 0.0);
        assert_eq!(ctrl.altitude, 10_000_000.0);
    }

    #[test]
    fn scroll_zoom() {
        let mut ctrl = GlobeController::default();
        let initial = ctrl.altitude;
        ctrl.handle_scroll(1.0);
        assert!(ctrl.altitude < initial);
        ctrl.handle_scroll(-1.0);
        assert!(ctrl.altitude > ctrl.min_altitude);
    }

    #[test]
    fn altitude_clamped() {
        let mut ctrl = GlobeController::default();
        ctrl.handle_scroll(100.0);
        assert!(ctrl.altitude >= ctrl.min_altitude);
        ctrl.altitude = ctrl.max_altitude;
        ctrl.handle_scroll(-100.0);
        assert!(ctrl.altitude <= ctrl.max_altitude);
    }

    #[test]
    fn latitude_clamped() {
        let mut ctrl = GlobeController::default();
        ctrl.latitude = 85.0;
        ctrl.handle_mouse_move(0.0, -100000.0);
        assert!(ctrl.latitude <= 85.0);
        assert!(ctrl.latitude >= -85.0);
    }

    #[test]
    fn camera_produces_finite_matrices() {
        let ctrl = GlobeController {
            latitude: 40.7128,
            longitude: -74.006,
            altitude: 100_000.0,
            ..GlobeController::default()
        };
        let cam = ctrl.camera();
        let vp = cam.view_projection();
        for col in 0..4 {
            for row in 0..4 {
                assert!(
                    vp.col(col)[row].is_finite(),
                    "VP matrix has non-finite value"
                );
            }
        }
    }

    #[test]
    fn tilt_clamped() {
        let mut ctrl = GlobeController::default();
        ctrl.handle_tilt(100.0);
        assert!(ctrl.tilt <= ctrl.max_tilt);
        ctrl.handle_tilt(-200.0);
        assert!(ctrl.tilt >= ctrl.min_tilt);
    }
}
