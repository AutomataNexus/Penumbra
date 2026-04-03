//! penumbra-atmosphere -- Atmospheric sky rendering for Penumbra.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use serde::{Deserialize, Serialize};

// ── Rayleigh config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RayleighConfig {
    /// Scattering coefficients (1/m) at sea level.
    pub scattering: [f32; 3],
    /// Scale height (m).
    pub scale_height: f32,
}

impl RayleighConfig {
    pub fn earth_default() -> Self {
        Self {
            scattering: [5.5e-6, 13.0e-6, 22.4e-6],
            scale_height: 8000.0,
        }
    }
}

// ── Mie config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MieConfig {
    /// Scattering coefficient at sea level.
    pub scattering: f32,
    /// Absorption coefficient.
    pub absorption: f32,
    /// Phase function asymmetry parameter (g).
    pub asymmetry: f32,
    /// Scale height (m).
    pub scale_height: f32,
}

impl MieConfig {
    pub fn earth_default() -> Self {
        Self {
            scattering: 21e-6,
            absorption: 4.4e-6,
            asymmetry: 0.758,
            scale_height: 1200.0,
        }
    }
}

// ── Atmosphere config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereConfig {
    /// Planet radius (m).
    pub planet_radius: f32,
    /// Atmosphere thickness (m).
    pub atmosphere_height: f32,
    pub rayleigh: RayleighConfig,
    pub mie: MieConfig,
    /// Sun intensity multiplier.
    pub sun_intensity: f32,
}

impl AtmosphereConfig {
    pub fn earth_default() -> Self {
        Self {
            planet_radius: 6_371_000.0,
            atmosphere_height: 100_000.0,
            rayleigh: RayleighConfig::earth_default(),
            mie: MieConfig::earth_default(),
            sun_intensity: 22.0,
        }
    }
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self::earth_default()
    }
}

// ── Fog ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FogMode {
    Linear,
    Exponential,
    ExponentialSquared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fog {
    pub mode: FogMode,
    pub color: [f32; 3],
    pub density: f32,
    pub start: f32,
    pub end: f32,
}

impl Default for Fog {
    fn default() -> Self {
        Self {
            mode: FogMode::Exponential,
            color: [0.7, 0.8, 0.9],
            density: 0.01,
            start: 10.0,
            end: 1000.0,
        }
    }
}

impl Fog {
    /// Compute the fog blending factor for a given distance.
    /// Returns 0.0 (no fog) to 1.0 (fully fogged).
    pub fn fog_factor(&self, distance: f32) -> f32 {
        let f = match self.mode {
            FogMode::Linear => {
                if self.end <= self.start {
                    1.0
                } else {
                    (self.end - distance) / (self.end - self.start)
                }
            }
            FogMode::Exponential => (-self.density * distance).exp(),
            FogMode::ExponentialSquared => {
                let d = self.density * distance;
                (-d * d).exp()
            }
        };
        f.clamp(0.0, 1.0)
    }
}

// ── Atmosphere renderer ──

pub struct AtmosphereRenderer {
    pub config: AtmosphereConfig,
    sun_direction: Vec3,
}

impl AtmosphereRenderer {
    pub fn new(config: AtmosphereConfig) -> Self {
        Self {
            config,
            sun_direction: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    pub fn sun_direction(&self) -> Vec3 {
        self.sun_direction
    }

    pub fn set_sun_direction(&mut self, dir: Vec3) {
        self.sun_direction = dir.normalize_or_zero();
    }

    /// Set the sun direction from an elevation angle in radians (0 = horizon, PI/2 = zenith).
    pub fn set_sun_elevation(&mut self, elevation_radians: f32) {
        let y = elevation_radians.sin();
        let xz = elevation_radians.cos();
        self.sun_direction = Vec3::new(xz, y, 0.0).normalize_or_zero();
    }

    pub fn to_uniform(&self) -> AtmosphereUniform {
        AtmosphereUniform {
            sun_direction: [
                self.sun_direction.x,
                self.sun_direction.y,
                self.sun_direction.z,
                0.0,
            ],
            rayleigh_scattering: [
                self.config.rayleigh.scattering[0],
                self.config.rayleigh.scattering[1],
                self.config.rayleigh.scattering[2],
                self.config.rayleigh.scale_height,
            ],
            mie_params: [
                self.config.mie.scattering,
                self.config.mie.absorption,
                self.config.mie.asymmetry,
                self.config.mie.scale_height,
            ],
            planet_params: [
                self.config.planet_radius,
                self.config.atmosphere_height,
                self.config.sun_intensity,
                0.0,
            ],
        }
    }
}

// ── Atmosphere uniform ──

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct AtmosphereUniform {
    pub sun_direction: [f32; 4],
    pub rayleigh_scattering: [f32; 4],
    pub mie_params: [f32; 4],
    pub planet_params: [f32; 4],
}

// ── Shader ──

pub const ATMOSPHERE_WGSL: &str = include_str!("shaders/atmosphere.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_defaults() {
        let cfg = AtmosphereConfig::earth_default();
        assert!((cfg.planet_radius - 6_371_000.0).abs() < 1.0);
        assert!((cfg.atmosphere_height - 100_000.0).abs() < 1.0);
        assert!((cfg.rayleigh.scale_height - 8000.0).abs() < 1.0);
        assert!((cfg.mie.asymmetry - 0.758).abs() < 0.001);
        assert!((cfg.sun_intensity - 22.0).abs() < 0.1);
    }

    #[test]
    fn fog_factor_exponential() {
        let fog = Fog {
            mode: FogMode::Exponential,
            density: 0.1,
            ..Default::default()
        };
        let f0 = fog.fog_factor(0.0);
        assert!((f0 - 1.0).abs() < 0.001);
        let f10 = fog.fog_factor(10.0);
        let expected = (-0.1_f32 * 10.0).exp();
        assert!((f10 - expected).abs() < 0.001);
    }

    #[test]
    fn fog_factor_linear() {
        let fog = Fog {
            mode: FogMode::Linear,
            start: 0.0,
            end: 100.0,
            ..Default::default()
        };
        let f50 = fog.fog_factor(50.0);
        assert!((f50 - 0.5).abs() < 0.001);
    }

    #[test]
    fn sun_elevation() {
        let mut renderer = AtmosphereRenderer::new(AtmosphereConfig::earth_default());
        // Zenith
        renderer.set_sun_elevation(std::f32::consts::FRAC_PI_2);
        let dir = renderer.sun_direction();
        assert!((dir.y - 1.0).abs() < 0.01);

        // Horizon
        renderer.set_sun_elevation(0.0);
        let dir = renderer.sun_direction();
        assert!((dir.y).abs() < 0.01);
        assert!((dir.x - 1.0).abs() < 0.01);
    }
}
