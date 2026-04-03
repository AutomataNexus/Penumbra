//! penumbra-post -- Post-processing pipeline for Penumbra.

use serde::{Deserialize, Serialize};

// ── PostPass trait ──

pub trait PostPass: std::fmt::Debug {
    fn name(&self) -> &str;
    fn enabled(&self) -> bool;
}

// ── Tone mapping ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToneMappingMode {
    ACES,
    Reinhard,
    Uncharted2,
    Linear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneMapping {
    pub mode: ToneMappingMode,
    pub exposure: f32,
    pub enabled: bool,
}

impl ToneMapping {
    pub fn aces() -> Self {
        Self {
            mode: ToneMappingMode::ACES,
            exposure: 1.0,
            enabled: true,
        }
    }

    pub fn reinhard() -> Self {
        Self {
            mode: ToneMappingMode::Reinhard,
            exposure: 1.0,
            enabled: true,
        }
    }
}

impl PostPass for ToneMapping {
    fn name(&self) -> &str {
        "ToneMapping"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── Bloom ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bloom {
    pub threshold: f32,
    pub intensity: f32,
    pub radius: f32,
    pub enabled: bool,
}

impl Default for Bloom {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            intensity: 0.5,
            radius: 5.0,
            enabled: true,
        }
    }
}

impl PostPass for Bloom {
    fn name(&self) -> &str {
        "Bloom"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── SSAO ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ssao {
    pub radius: f32,
    pub bias: f32,
    pub intensity: f32,
    pub samples: u32,
    pub enabled: bool,
}

impl Default for Ssao {
    fn default() -> Self {
        Self {
            radius: 0.5,
            bias: 0.025,
            intensity: 1.0,
            samples: 32,
            enabled: true,
        }
    }
}

impl PostPass for Ssao {
    fn name(&self) -> &str {
        "SSAO"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── FXAA ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fxaa {
    pub edge_threshold: f32,
    pub edge_threshold_min: f32,
    pub enabled: bool,
}

impl Default for Fxaa {
    fn default() -> Self {
        Self {
            edge_threshold: 0.166,
            edge_threshold_min: 0.0833,
            enabled: true,
        }
    }
}

impl PostPass for Fxaa {
    fn name(&self) -> &str {
        "FXAA"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── Color grading ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGrading {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub enabled: bool,
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            enabled: true,
        }
    }
}

impl PostPass for ColorGrading {
    fn name(&self) -> &str {
        "ColorGrading"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── Vignette ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vignette {
    pub intensity: f32,
    pub radius: f32,
    pub softness: f32,
    pub enabled: bool,
}

impl Default for Vignette {
    fn default() -> Self {
        Self {
            intensity: 0.5,
            radius: 0.8,
            softness: 0.3,
            enabled: true,
        }
    }
}

impl PostPass for Vignette {
    fn name(&self) -> &str {
        "Vignette"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── Chromatic aberration ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaticAberration {
    pub intensity: f32,
    pub enabled: bool,
}

impl Default for ChromaticAberration {
    fn default() -> Self {
        Self {
            intensity: 0.005,
            enabled: true,
        }
    }
}

impl PostPass for ChromaticAberration {
    fn name(&self) -> &str {
        "ChromaticAberration"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── Sharpen ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sharpen {
    pub strength: f32,
    pub enabled: bool,
}

impl Default for Sharpen {
    fn default() -> Self {
        Self {
            strength: 0.5,
            enabled: true,
        }
    }
}

impl PostPass for Sharpen {
    fn name(&self) -> &str {
        "Sharpen"
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── Post pipeline ──

pub struct PostPipeline {
    passes: Vec<Box<dyn PostPass>>,
}

impl PostPipeline {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Builder pattern: add a pass and return self.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, pass: impl PostPass + 'static) -> Self {
        self.passes.push(Box::new(pass));
        self
    }

    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    pub fn passes(&self) -> &[Box<dyn PostPass>] {
        &self.passes
    }

    pub fn enabled_passes(&self) -> Vec<&dyn PostPass> {
        self.passes
            .iter()
            .filter(|p| p.enabled())
            .map(|p| p.as_ref())
            .collect()
    }
}

impl Default for PostPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ── Shaders ──

pub const TONE_MAPPING_WGSL: &str = include_str!("shaders/tone_mapping.wgsl");
pub const FXAA_WGSL: &str = include_str!("shaders/fxaa.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_builder() {
        let pipeline = PostPipeline::new()
            .add(ToneMapping::aces())
            .add(Bloom::default())
            .add(Fxaa::default());
        assert_eq!(pipeline.pass_count(), 3);
    }

    #[test]
    fn pass_count() {
        let pipeline = PostPipeline::new()
            .add(Ssao::default())
            .add(ColorGrading::default());
        assert_eq!(pipeline.pass_count(), 2);
        assert_eq!(pipeline.enabled_passes().len(), 2);
    }

    #[test]
    fn tone_mapping_aces() {
        let tm = ToneMapping::aces();
        assert_eq!(tm.mode, ToneMappingMode::ACES);
        assert_eq!(tm.exposure, 1.0);
        assert!(tm.enabled);
        assert_eq!(tm.name(), "ToneMapping");
    }

    #[test]
    fn disabled_pass_filtered() {
        let mut fxaa = Fxaa::default();
        fxaa.enabled = false;
        let pipeline = PostPipeline::new().add(ToneMapping::aces()).add(fxaa);
        assert_eq!(pipeline.pass_count(), 2);
        assert_eq!(pipeline.enabled_passes().len(), 1);
    }
}
