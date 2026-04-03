use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_identity() {
        let t = Transform::default();
        let m = t.to_matrix();
        assert_eq!(m, Mat4::IDENTITY);
    }

    #[test]
    fn translation_matrix() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let m = t.to_matrix();
        let expected = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(m, expected);
    }

    #[test]
    fn scale_matrix() {
        let t = Transform {
            scale: Vec3::splat(2.0),
            ..Default::default()
        };
        let m = t.to_matrix();
        let expected = Mat4::from_scale(Vec3::splat(2.0));
        assert_eq!(m, expected);
    }
}
