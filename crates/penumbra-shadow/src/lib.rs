//! penumbra-shadow -- Shadow mapping for Penumbra.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

// ── Config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowConfig {
    pub cascades: u32,
    pub map_size: u32,
    pub pcf: bool,
    pub bias: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            cascades: 4,
            map_size: 2048,
            pcf: true,
            bias: 0.005,
        }
    }
}

// ── Shadow uniform ──

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ShadowUniform {
    pub light_space_matrices: [[f32; 16]; 4],
    pub cascade_splits: [f32; 4],
    pub shadow_map_size: f32,
    pub bias: f32,
    pub pcf_enabled: f32,
    pub cascade_count: f32,
}

// ── Cascade shadow map ──

pub struct CascadeShadowMap {
    pub config: ShadowConfig,
    pub cascade_splits: Vec<f32>,
    pub light_space_matrices: Vec<Mat4>,
}

impl CascadeShadowMap {
    pub fn new(config: ShadowConfig) -> Self {
        let n = config.cascades as usize;
        Self {
            cascade_splits: vec![0.0; n],
            light_space_matrices: vec![Mat4::IDENTITY; n],
            config,
        }
    }

    pub fn cascade_count(&self) -> u32 {
        self.config.cascades
    }

    /// Update cascade splits and matrices for a directional light.
    pub fn update(
        &mut self,
        light_dir: Vec3,
        near: f32,
        far: f32,
        _view: Mat4,
        _projection: Mat4,
    ) {
        let n = self.config.cascades as usize;
        // Compute cascade splits using practical split scheme (logarithmic)
        let lambda = 0.5_f32;
        self.cascade_splits.resize(n, 0.0);
        for i in 0..n {
            let p = (i + 1) as f32 / n as f32;
            let log_split = near * (far / near).powf(p);
            let uniform_split = near + (far - near) * p;
            self.cascade_splits[i] = lambda * log_split + (1.0 - lambda) * uniform_split;
        }

        // Compute light-space matrices for each cascade
        self.light_space_matrices.resize(n, Mat4::IDENTITY);
        let light_dir_norm = light_dir.normalize_or_zero();

        for i in 0..n {
            let cascade_near = if i == 0 { near } else { self.cascade_splits[i - 1] };
            let cascade_far = self.cascade_splits[i];

            // Simple orthographic projection for the cascade
            let center = Vec3::ZERO;
            let extent = cascade_far - cascade_near;
            let light_view = Mat4::look_at_rh(
                center - light_dir_norm * extent,
                center,
                Vec3::Y,
            );
            let light_proj = Mat4::orthographic_rh(
                -extent, extent, -extent, extent, 0.0, extent * 2.0,
            );
            self.light_space_matrices[i] = light_proj * light_view;
        }
    }
}

// ── Point shadow map ──

pub struct PointShadowMap {
    pub map_size: u32,
    pub near: f32,
    pub far: f32,
}

impl PointShadowMap {
    pub fn new(map_size: u32, near: f32, far: f32) -> Self {
        Self {
            map_size,
            near,
            far,
        }
    }

    /// Return the 6 view-projection matrices for a cubemap shadow from a point light.
    pub fn face_view_projections(&self, light_pos: Vec3) -> [Mat4; 6] {
        let proj = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_2,
            1.0,
            self.near,
            self.far,
        );
        let faces = [
            (Vec3::X, Vec3::NEG_Y),   // +X
            (Vec3::NEG_X, Vec3::NEG_Y), // -X
            (Vec3::Y, Vec3::Z),        // +Y
            (Vec3::NEG_Y, Vec3::NEG_Z), // -Y
            (Vec3::Z, Vec3::NEG_Y),    // +Z
            (Vec3::NEG_Z, Vec3::NEG_Y), // -Z
        ];
        let mut result = [Mat4::IDENTITY; 6];
        for (i, (dir, up)) in faces.iter().enumerate() {
            let view = Mat4::look_at_rh(light_pos, light_pos + *dir, *up);
            result[i] = proj * view;
        }
        result
    }
}

// ── Shader ──

pub const SHADOW_WGSL: &str = include_str!("shaders/shadow.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = ShadowConfig::default();
        assert_eq!(cfg.cascades, 4);
        assert_eq!(cfg.map_size, 2048);
        assert!(cfg.pcf);
        assert!((cfg.bias - 0.005).abs() < 1e-6);
    }

    #[test]
    fn cascade_count() {
        let csm = CascadeShadowMap::new(ShadowConfig::default());
        assert_eq!(csm.cascade_count(), 4);
    }

    #[test]
    fn point_shadow_6_faces() {
        let psm = PointShadowMap::new(1024, 0.1, 100.0);
        let vps = psm.face_view_projections(Vec3::ZERO);
        assert_eq!(vps.len(), 6);
        // Each should be a valid (non-identity) matrix
        for vp in &vps {
            assert_ne!(*vp, Mat4::IDENTITY);
        }
    }

    #[test]
    fn cascade_update_produces_valid_matrices() {
        let mut csm = CascadeShadowMap::new(ShadowConfig {
            cascades: 3,
            ..Default::default()
        });
        csm.update(
            Vec3::new(0.0, -1.0, -1.0),
            0.1,
            100.0,
            Mat4::IDENTITY,
            Mat4::IDENTITY,
        );
        assert_eq!(csm.cascade_splits.len(), 3);
        assert_eq!(csm.light_space_matrices.len(), 3);
        // Splits should be monotonically increasing
        for i in 1..csm.cascade_splits.len() {
            assert!(csm.cascade_splits[i] > csm.cascade_splits[i - 1]);
        }
    }
}
