use glam::Vec3;
use penumbra_backend::Rgb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Light {
    Directional {
        direction: Vec3,
        color: Rgb,
        intensity: f32,
        cast_shadows: bool,
    },
    Point {
        position: Vec3,
        color: Rgb,
        intensity: f32,
        range: f32,
        cast_shadows: bool,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        color: Rgb,
        intensity: f32,
        range: f32,
        inner_angle: f32,
        outer_angle: f32,
        cast_shadows: bool,
    },
}
