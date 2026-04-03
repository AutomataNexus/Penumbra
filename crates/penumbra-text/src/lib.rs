//! penumbra-text -- SDF text rendering for Penumbra.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Font ID ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontId(pub u64);

// ── Font descriptor ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontDescriptor {
    pub id: FontId,
    pub name: String,
    pub size: f32,
    pub sdf_spread: f32,
}

// ── Glyph metrics ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlyphMetrics {
    pub codepoint: char,
    pub advance: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub width: f32,
    pub height: f32,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

// ── Font atlas ──

#[derive(Debug, Clone)]
pub struct FontAtlas {
    pub font_id: FontId,
    pub atlas_width: u32,
    pub atlas_height: u32,
    glyphs: HashMap<char, GlyphMetrics>,
}

impl FontAtlas {
    pub fn new(font_id: FontId, width: u32, height: u32) -> Self {
        Self {
            font_id,
            atlas_width: width,
            atlas_height: height,
            glyphs: HashMap::new(),
        }
    }

    pub fn add_glyph(&mut self, metrics: GlyphMetrics) {
        self.glyphs.insert(metrics.codepoint, metrics);
    }

    pub fn get_glyph(&self, ch: char) -> Option<&GlyphMetrics> {
        self.glyphs.get(&ch)
    }

    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }
}

// ── Text layout ──

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub metrics: GlyphMetrics,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
}

/// Lay out a string of text using the given font atlas, returning positioned glyphs.
pub fn layout_text(atlas: &FontAtlas, text: &str, font_size: f32) -> TextLayout {
    let mut x = 0.0_f32;
    let mut glyphs = Vec::new();
    let mut max_height = 0.0_f32;

    for ch in text.chars() {
        if let Some(metrics) = atlas.get_glyph(ch) {
            let scale = font_size / 32.0; // assume metrics are at 32px base
            glyphs.push(PositionedGlyph {
                metrics: *metrics,
                x: x + metrics.bearing_x * scale,
                y: metrics.bearing_y * scale,
            });
            x += metrics.advance * scale;
            max_height = max_height.max(metrics.height * scale);
        }
    }

    TextLayout {
        width: x,
        height: max_height,
        glyphs,
    }
}

// ── Text anchor ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

// ── Billboard text ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillboardMode {
    ScreenAligned,
    AxisAligned,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillboardText {
    pub text: String,
    pub font_id: FontId,
    pub font_size: f32,
    pub color: [f32; 4],
    pub position: [f32; 3],
    pub billboard_mode: BillboardMode,
    pub anchor: TextAnchor,
}

// ── 2D text ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text2d {
    pub text: String,
    pub font_id: FontId,
    pub font_size: f32,
    pub color: [f32; 4],
    pub position: [f32; 2],
    pub anchor: TextAnchor,
}

// ── Glyph vertex ──

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GlyphVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

// ── Text batch ──

#[derive(Debug, Clone, Default)]
pub struct TextBatch {
    pub vertices: Vec<GlyphVertex>,
    pub indices: Vec<u32>,
}

impl TextBatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a positioned layout to this batch at a given z depth.
    pub fn add_layout(&mut self, layout: &TextLayout, z: f32, color: [f32; 4]) {
        for glyph in &layout.glyphs {
            let base = self.vertices.len() as u32;
            let m = &glyph.metrics;
            let x0 = glyph.x;
            let y0 = glyph.y;
            let x1 = x0 + m.width;
            let y1 = y0 + m.height;

            self.vertices.push(GlyphVertex {
                position: [x0, y0, z],
                uv: [m.uv_min[0], m.uv_min[1]],
                color,
            });
            self.vertices.push(GlyphVertex {
                position: [x1, y0, z],
                uv: [m.uv_max[0], m.uv_min[1]],
                color,
            });
            self.vertices.push(GlyphVertex {
                position: [x1, y1, z],
                uv: [m.uv_max[0], m.uv_max[1]],
                color,
            });
            self.vertices.push(GlyphVertex {
                position: [x0, y1, z],
                uv: [m.uv_min[0], m.uv_max[1]],
                color,
            });

            self.indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

// ── Shader ──

pub const SDF_TEXT_WGSL: &str = include_str!("shaders/sdf_text.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    fn make_atlas() -> FontAtlas {
        let mut atlas = FontAtlas::new(FontId(0), 512, 512);
        atlas.add_glyph(GlyphMetrics {
            codepoint: 'A',
            advance: 20.0,
            bearing_x: 1.0,
            bearing_y: 24.0,
            width: 18.0,
            height: 24.0,
            uv_min: [0.0, 0.0],
            uv_max: [0.1, 0.1],
        });
        atlas.add_glyph(GlyphMetrics {
            codepoint: 'B',
            advance: 20.0,
            bearing_x: 2.0,
            bearing_y: 24.0,
            width: 16.0,
            height: 24.0,
            uv_min: [0.1, 0.0],
            uv_max: [0.2, 0.1],
        });
        atlas
    }

    #[test]
    fn font_atlas_add_get_glyph() {
        let atlas = make_atlas();
        assert_eq!(atlas.glyph_count(), 2);
        let g = atlas.get_glyph('A').unwrap();
        assert_eq!(g.codepoint, 'A');
        assert!(atlas.get_glyph('Z').is_none());
    }

    #[test]
    fn layout_positions_glyphs() {
        let atlas = make_atlas();
        let layout = layout_text(&atlas, "AB", 32.0);
        assert_eq!(layout.glyphs.len(), 2);
        // First glyph at x = bearing_x
        assert!((layout.glyphs[0].x - 1.0).abs() < 0.01);
        // Second glyph shifted by first advance
        assert!(layout.glyphs[1].x > layout.glyphs[0].x);
        assert!(layout.width > 0.0);
    }

    #[test]
    fn batch_vertex_counts() {
        let atlas = make_atlas();
        let layout = layout_text(&atlas, "AB", 32.0);
        let mut batch = TextBatch::new();
        batch.add_layout(&layout, 0.0, [1.0; 4]);
        // 2 glyphs * 4 vertices each = 8
        assert_eq!(batch.vertex_count(), 8);
        // 2 glyphs * 6 indices each = 12
        assert_eq!(batch.index_count(), 12);
    }
}
