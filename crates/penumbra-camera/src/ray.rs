//! Ray casting utilities — screen-to-ray, plane and AABB intersection.

use glam::{Mat4, Vec2, Vec3, Vec4};
use penumbra_core::Aabb;
use serde::{Deserialize, Serialize};

/// A ray defined by an origin and a normalised direction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray. `direction` is normalised internally.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Point along the ray at parameter `t`.
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Intersect with a plane defined by `normal . P = d`.
    /// Returns `Some(t)` if the ray intersects, `None` if parallel.
    pub fn intersect_plane(&self, normal: Vec3, d: f32) -> Option<f32> {
        let denom = normal.dot(self.direction);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = (d - normal.dot(self.origin)) / denom;
        Some(t)
    }

    /// Intersect with an axis-aligned bounding box using the slab method.
    /// Returns `Some(t)` for the nearest positive intersection, or `None`.
    ///
    /// This uses the standard IEEE 754 slab method where `1.0 / 0.0 = ±inf`
    /// naturally handles axis-aligned rays. The only degenerate case (origin
    /// exactly on a slab boundary with zero direction component) may produce NaN,
    /// which correctly falls through to `None`.
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let inv_dir = Vec3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (aabb.min.x - self.origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - self.origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - self.origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - self.origin.y) * inv_dir.y;
        let t5 = (aabb.min.z - self.origin.z) * inv_dir.z;
        let t6 = (aabb.max.z - self.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax || tmin.is_nan() || tmax.is_nan() {
            return None;
        }

        // If tmin < 0 the ray origin is inside the AABB; return tmax.
        Some(if tmin < 0.0 { tmax } else { tmin })
    }
}

/// Convert a screen-space position to a world-space ray.
///
/// `screen_pos` is in pixels `[0, viewport_size.x) x [0, viewport_size.y)`.
/// `inv_view_proj` is the inverse of the combined view-projection matrix.
pub fn screen_to_ray(screen_pos: Vec2, viewport_size: Vec2, inv_view_proj: Mat4) -> Ray {
    // Convert to NDC [-1, 1].
    let ndc_x = (screen_pos.x / viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (screen_pos.y / viewport_size.y) * 2.0; // flip Y

    let near_ndc = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let far_ndc = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

    let near_world = inv_view_proj * near_ndc;
    let far_world = inv_view_proj * far_ndc;

    let near_world = near_world.truncate() / near_world.w;
    let far_world = far_world.truncate() / far_world.w;

    Ray::new(near_world, far_world - near_world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_ray_point_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let p = ray.point_at(5.0);
        assert!((p - Vec3::new(0.0, 0.0, 5.0)).length() < 1e-5);
    }

    #[test]
    fn test_ray_intersect_plane() {
        // XZ plane at y=0: normal = Y, d = 0
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let t = ray.intersect_plane(Vec3::Y, 0.0).unwrap();
        assert!((t - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_intersect_plane_parallel() {
        let ray = Ray::new(Vec3::new(0.0, 1.0, 0.0), Vec3::X);
        assert!(ray.intersect_plane(Vec3::Y, 0.0).is_none());
    }

    #[test]
    fn test_ray_intersect_aabb() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let t = ray.intersect_aabb(&aabb).unwrap();
        assert!((t - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_miss_aabb() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(1.0, 0.0, 0.0));
        assert!(ray.intersect_aabb(&aabb).is_none());
    }

    #[test]
    fn test_screen_to_ray_center() {
        use crate::camera::PerspectiveCamera;

        let cam = PerspectiveCamera {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: 60.0,
            near: 0.1,
            far: 100.0,
            aspect: 1.0,
        };

        let vp = cam.view_projection();
        let inv_vp = vp.inverse();
        let viewport = Vec2::new(800.0, 600.0);
        let center = Vec2::new(400.0, 300.0);
        let ray = screen_to_ray(center, viewport, inv_vp);

        // The center ray should point roughly from camera position towards target (negative Z).
        assert!(ray.direction.z < 0.0, "ray should point towards -Z");
        // Origin should be near the camera position (near plane).
        assert!((ray.origin - cam.position).length() < 1.0);
    }
}
