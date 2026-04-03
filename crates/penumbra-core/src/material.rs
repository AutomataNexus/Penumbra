use penumbra_backend::{Rgb, Rgba, TextureId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialId(pub u64);

impl MaterialId {
    /// Sentinel value indicating an unassigned material. The renderer never produces this ID.
    pub const INVALID: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask {
        cutoff: f32,
    },
    Blend,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub id: MaterialId,
    pub albedo: Rgba,
    pub albedo_texture: Option<TextureId>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<TextureId>,
    pub normal_texture: Option<TextureId>,
    pub emissive: Rgb,
    pub emissive_texture: Option<TextureId>,
    pub occlusion_texture: Option<TextureId>,
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            id: MaterialId::INVALID,
            albedo: Rgba::WHITE,
            albedo_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            metallic_roughness_texture: None,
            normal_texture: None,
            emissive: Rgb::BLACK,
            emissive_texture: None,
            occlusion_texture: None,
            alpha_mode: AlphaMode::default(),
            double_sided: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_material() {
        let mat = Material::default();
        assert_eq!(mat.albedo, Rgba::WHITE);
        assert_eq!(mat.metallic, 0.0);
        assert_eq!(mat.roughness, 1.0);
        assert_eq!(mat.alpha_mode, AlphaMode::Opaque);
        assert!(!mat.double_sided);
    }
}
