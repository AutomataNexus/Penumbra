use glam::{Mat4, Vec3, Vec4};
use penumbra_backend::Aabb;

#[derive(Debug, Clone)]
pub struct Frustum {
    planes: [Vec4; 6], // normal.xyz + distance in w
}

impl Frustum {
    /// Extract frustum planes from a view-projection matrix (Gribb/Hartmann method).
    pub fn from_view_projection(vp: Mat4) -> Self {
        let row0 = Vec4::new(vp.x_axis.x, vp.y_axis.x, vp.z_axis.x, vp.w_axis.x);
        let row1 = Vec4::new(vp.x_axis.y, vp.y_axis.y, vp.z_axis.y, vp.w_axis.y);
        let row2 = Vec4::new(vp.x_axis.z, vp.y_axis.z, vp.z_axis.z, vp.w_axis.z);
        let row3 = Vec4::new(vp.x_axis.w, vp.y_axis.w, vp.z_axis.w, vp.w_axis.w);

        let mut planes = [
            row3 + row0, // left
            row3 - row0, // right
            row3 + row1, // bottom
            row3 - row1, // top
            row2,        // near (RH: row3 + row2 for LH)
            row3 - row2, // far
        ];

        // Normalize planes
        for plane in &mut planes {
            let len = Vec3::new(plane.x, plane.y, plane.z).length();
            if len > 0.0 {
                *plane /= len;
            }
        }

        Self { planes }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            let dist = plane.x * point.x + plane.y * point.y + plane.z * point.z + plane.w;
            if dist < 0.0 {
                return false;
            }
        }
        true
    }

    pub fn contains_aabb(&self, aabb: &Aabb, transform: &Mat4) -> bool {
        let corners = [
            *transform * Vec4::new(aabb.min.x, aabb.min.y, aabb.min.z, 1.0),
            *transform * Vec4::new(aabb.max.x, aabb.min.y, aabb.min.z, 1.0),
            *transform * Vec4::new(aabb.min.x, aabb.max.y, aabb.min.z, 1.0),
            *transform * Vec4::new(aabb.max.x, aabb.max.y, aabb.min.z, 1.0),
            *transform * Vec4::new(aabb.min.x, aabb.min.y, aabb.max.z, 1.0),
            *transform * Vec4::new(aabb.max.x, aabb.min.y, aabb.max.z, 1.0),
            *transform * Vec4::new(aabb.min.x, aabb.max.y, aabb.max.z, 1.0),
            *transform * Vec4::new(aabb.max.x, aabb.max.y, aabb.max.z, 1.0),
        ];

        for plane in &self.planes {
            let mut all_outside = true;
            for corner in &corners {
                let dist = plane.x * corner.x + plane.y * corner.y + plane.z * corner.z + plane.w;
                if dist >= 0.0 {
                    all_outside = false;
                    break;
                }
            }
            if all_outside {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frustum() -> Frustum {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60_f32.to_radians(), 1.0, 0.1, 100.0);
        Frustum::from_view_projection(proj * view)
    }

    #[test]
    fn point_in_front_of_camera() {
        let frustum = test_frustum();
        assert!(frustum.contains_point(Vec3::ZERO));
    }

    #[test]
    fn point_behind_camera() {
        let frustum = test_frustum();
        assert!(!frustum.contains_point(Vec3::new(0.0, 0.0, 10.0)));
    }

    #[test]
    fn aabb_in_view() {
        let frustum = test_frustum();
        let aabb = Aabb::new(Vec3::splat(-0.5), Vec3::splat(0.5));
        assert!(frustum.contains_aabb(&aabb, &Mat4::IDENTITY));
    }

    #[test]
    fn aabb_behind_camera() {
        let frustum = test_frustum();
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, 8.0), Vec3::new(1.0, 1.0, 10.0));
        assert!(!frustum.contains_aabb(&aabb, &Mat4::IDENTITY));
    }
}
