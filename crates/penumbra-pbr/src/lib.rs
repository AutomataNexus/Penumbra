//! penumbra-pbr -- PBR rendering pipeline for Penumbra.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

// ── Light ──

/// A light source in the scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Light {
    Directional {
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        shadows: bool,
    },
    Point {
        position: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        range: f32,
        shadows: bool,
    },
    Spot {
        position: [f32; 3],
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        range: f32,
        inner_cone: f32,
        outer_cone: f32,
        shadows: bool,
    },
}

impl Light {
    /// Convert to a GPU-uploadable uniform.
    pub fn to_uniform(&self) -> LightUniform {
        match self {
            Light::Directional {
                direction,
                color,
                intensity,
                shadows,
            } => LightUniform {
                position: [0.0, 0.0, 0.0, 0.0],
                direction: [direction[0], direction[1], direction[2], 0.0],
                color: [color[0], color[1], color[2], *intensity],
                params: [
                    0.0, // type: directional
                    0.0,
                    0.0,
                    if *shadows { 1.0 } else { 0.0 },
                ],
            },
            Light::Point {
                position,
                color,
                intensity,
                range,
                shadows,
            } => LightUniform {
                position: [position[0], position[1], position[2], 1.0],
                direction: [0.0, 0.0, 0.0, 0.0],
                color: [color[0], color[1], color[2], *intensity],
                params: [
                    1.0, // type: point
                    *range,
                    0.0,
                    if *shadows { 1.0 } else { 0.0 },
                ],
            },
            Light::Spot {
                position,
                direction,
                color,
                intensity,
                range,
                inner_cone,
                outer_cone,
                shadows,
            } => LightUniform {
                position: [position[0], position[1], position[2], 1.0],
                direction: [
                    direction[0],
                    direction[1],
                    direction[2],
                    if *shadows { 1.0 } else { 0.0 },
                ],
                color: [color[0], color[1], color[2], *intensity],
                params: [
                    2.0, // type: spot
                    *range,
                    inner_cone.cos(),
                    outer_cone.cos(),
                ],
            },
        }
    }
}

// ── GPU uniforms ──

/// GPU-uploadable light data (64 bytes).
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LightUniform {
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub params: [f32; 4],
}

/// GPU-uploadable material data (48 bytes).
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct MaterialUniform {
    pub albedo: [f32; 4],
    pub emissive: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_cutoff: f32,
    pub _padding: f32,
}

impl Default for MaterialUniform {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0, 1.0],
            emissive: [0.0, 0.0, 0.0, 0.0],
            metallic: 0.0,
            roughness: 1.0,
            alpha_cutoff: 0.5,
            _padding: 0.0,
        }
    }
}

// ── Environment config ──

/// IBL / environment settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub intensity: f32,
    pub rotation: f32,
    pub diffuse_only: bool,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            rotation: 0.0,
            diffuse_only: false,
        }
    }
}

// ── PBR config ──

/// Top-level PBR pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrConfig {
    pub max_lights: usize,
    pub enable_ibl: bool,
    pub shadow_mapping: bool,
    pub environment: EnvironmentConfig,
}

impl Default for PbrConfig {
    fn default() -> Self {
        Self {
            max_lights: 16,
            enable_ibl: true,
            shadow_mapping: true,
            environment: EnvironmentConfig::default(),
        }
    }
}

// ── PBR pipeline ──

/// Manages lights, environment, and pipeline state for PBR rendering.
pub struct PbrPipeline {
    pub config: PbrConfig,
    lights: Vec<Light>,
}

impl PbrPipeline {
    pub fn new(config: PbrConfig) -> Self {
        Self {
            config,
            lights: Vec::new(),
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }

    pub fn lights(&self) -> &[Light] {
        &self.lights
    }

    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// Build an array of GPU light uniforms.
    pub fn light_uniforms(&self) -> Vec<LightUniform> {
        self.lights.iter().map(|l| l.to_uniform()).collect()
    }
}

// ── Shaders (embedded as const strings) ──

pub const PBR_VERTEX_WGSL: &str = include_str!("shaders/pbr.wgsl");
pub const LIGHTING_WGSL: &str = include_str!("shaders/lighting.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_uniform_directional() {
        let light = Light::Directional {
            direction: [0.0, -1.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 2.0,
            shadows: true,
        };
        let u = light.to_uniform();
        assert_eq!(u.position, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(u.direction[1], -1.0);
        assert_eq!(u.color[3], 2.0);
        assert_eq!(u.params[0], 0.0);
        assert_eq!(u.params[3], 1.0);
    }

    #[test]
    fn light_uniform_point() {
        let light = Light::Point {
            position: [1.0, 2.0, 3.0],
            color: [1.0, 0.5, 0.0],
            intensity: 5.0,
            range: 10.0,
            shadows: false,
        };
        let u = light.to_uniform();
        assert_eq!(u.position[0], 1.0);
        assert_eq!(u.params[0], 1.0);
        assert_eq!(u.params[1], 10.0);
    }

    #[test]
    fn default_config() {
        let cfg = PbrConfig::default();
        assert_eq!(cfg.max_lights, 16);
        assert!(cfg.enable_ibl);
        assert!(cfg.shadow_mapping);
        assert_eq!(cfg.environment.intensity, 1.0);
        assert!(!cfg.environment.diffuse_only);
    }

    #[test]
    fn pipeline_add_clear_lights() {
        let mut pipeline = PbrPipeline::new(PbrConfig::default());
        assert_eq!(pipeline.light_count(), 0);

        pipeline.add_light(Light::Directional {
            direction: [0.0, -1.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            shadows: false,
        });
        pipeline.add_light(Light::Point {
            position: [0.0, 5.0, 0.0],
            color: [1.0, 0.0, 0.0],
            intensity: 3.0,
            range: 20.0,
            shadows: true,
        });
        assert_eq!(pipeline.light_count(), 2);
        assert_eq!(pipeline.light_uniforms().len(), 2);

        pipeline.clear_lights();
        assert_eq!(pipeline.light_count(), 0);
    }

    #[test]
    fn material_uniform_default() {
        let u = MaterialUniform::default();
        assert_eq!(u.albedo, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(u.metallic, 0.0);
        assert_eq!(u.roughness, 1.0);
    }
}
