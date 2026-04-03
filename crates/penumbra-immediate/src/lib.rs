//! penumbra-immediate -- Immediate mode debug rendering for Penumbra.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

// ── Vertex ──

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ImmediateVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

// ── Billboard ──

#[derive(Debug, Clone)]
pub struct BillboardDesc {
    pub position: Vec3,
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

// ── Batch ──

#[derive(Debug, Clone, Default)]
pub struct ImmediateBatch {
    pub line_vertices: Vec<ImmediateVertex>,
    pub triangle_vertices: Vec<ImmediateVertex>,
    pub triangle_indices: Vec<u32>,
}

// ── Renderer ──

pub struct ImmediateRenderer {
    batch: ImmediateBatch,
}

impl ImmediateRenderer {
    pub fn new() -> Self {
        Self {
            batch: ImmediateBatch::default(),
        }
    }

    pub fn batch(&self) -> &ImmediateBatch {
        &self.batch
    }

    pub fn clear(&mut self) {
        self.batch.line_vertices.clear();
        self.batch.triangle_vertices.clear();
        self.batch.triangle_indices.clear();
    }

    /// Draw a single line segment.
    pub fn draw_line(&mut self, a: Vec3, b: Vec3, color: [f32; 4]) {
        let va = ImmediateVertex {
            position: a.into(),
            color,
            uv: [0.0, 0.0],
        };
        let vb = ImmediateVertex {
            position: b.into(),
            color,
            uv: [1.0, 0.0],
        };
        self.batch.line_vertices.push(va);
        self.batch.line_vertices.push(vb);
    }

    /// Draw a connected polyline.
    pub fn draw_polyline(&mut self, points: &[Vec3], color: [f32; 4]) {
        for pair in points.windows(2) {
            self.draw_line(pair[0], pair[1], color);
        }
    }

    /// Draw a wireframe box (12 edges = 24 line vertices).
    pub fn draw_box(&mut self, min: Vec3, max: Vec3, color: [f32; 4]) {
        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ];
        // Bottom face
        self.draw_line(corners[0], corners[1], color);
        self.draw_line(corners[1], corners[2], color);
        self.draw_line(corners[2], corners[3], color);
        self.draw_line(corners[3], corners[0], color);
        // Top face
        self.draw_line(corners[4], corners[5], color);
        self.draw_line(corners[5], corners[6], color);
        self.draw_line(corners[6], corners[7], color);
        self.draw_line(corners[7], corners[4], color);
        // Verticals
        self.draw_line(corners[0], corners[4], color);
        self.draw_line(corners[1], corners[5], color);
        self.draw_line(corners[2], corners[6], color);
        self.draw_line(corners[3], corners[7], color);
    }

    /// Draw a wireframe sphere approximation using 3 circles.
    pub fn draw_sphere(&mut self, center: Vec3, radius: f32, color: [f32; 4]) {
        let segments = 16;
        for ring in 0..3 {
            let mut points = Vec::with_capacity(segments + 1);
            for i in 0..=segments {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                let (s, c) = angle.sin_cos();
                let p = match ring {
                    0 => center + Vec3::new(c * radius, s * radius, 0.0), // XY
                    1 => center + Vec3::new(c * radius, 0.0, s * radius), // XZ
                    _ => center + Vec3::new(0.0, c * radius, s * radius), // YZ
                };
                points.push(p);
            }
            self.draw_polyline(&points, color);
        }
    }

    /// Draw an arrow from `start` to `end`.
    pub fn draw_arrow(&mut self, start: Vec3, end: Vec3, color: [f32; 4]) {
        self.draw_line(start, end, color);
        let dir = (end - start).normalize_or_zero();
        let len = (end - start).length();
        let head_len = len * 0.15;
        // Create two perpendicular vectors for the arrowhead
        let up = if dir.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
        let right = dir.cross(up).normalize_or_zero() * head_len * 0.5;
        let head_base = end - dir * head_len;
        self.draw_line(end, head_base + right, color);
        self.draw_line(end, head_base - right, color);
    }

    /// Draw an AABB (alias for draw_box).
    pub fn draw_aabb(&mut self, min: Vec3, max: Vec3, color: [f32; 4]) {
        self.draw_box(min, max, color);
    }

    /// Draw a grid on the XZ plane.
    pub fn draw_grid(&mut self, size: f32, divisions: u32, color: [f32; 4]) {
        let half = size / 2.0;
        let step = size / divisions as f32;
        for i in 0..=divisions {
            let offset = -half + step * i as f32;
            // Lines along Z
            self.draw_line(
                Vec3::new(offset, 0.0, -half),
                Vec3::new(offset, 0.0, half),
                color,
            );
            // Lines along X
            self.draw_line(
                Vec3::new(-half, 0.0, offset),
                Vec3::new(half, 0.0, offset),
                color,
            );
        }
    }

    /// Draw a filled rectangle as two triangles.
    pub fn draw_filled_rect(&mut self, min: Vec3, max: Vec3, color: [f32; 4]) {
        let base = self.batch.triangle_vertices.len() as u32;
        let corners = [
            ImmediateVertex {
                position: [min.x, min.y, min.z],
                color,
                uv: [0.0, 0.0],
            },
            ImmediateVertex {
                position: [max.x, min.y, min.z],
                color,
                uv: [1.0, 0.0],
            },
            ImmediateVertex {
                position: [max.x, max.y, max.z],
                color,
                uv: [1.0, 1.0],
            },
            ImmediateVertex {
                position: [min.x, max.y, max.z],
                color,
                uv: [0.0, 1.0],
            },
        ];
        self.batch.triangle_vertices.extend_from_slice(&corners);
        self.batch.triangle_indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }
}

impl Default for ImmediateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_line_adds_2_verts() {
        let mut r = ImmediateRenderer::new();
        r.draw_line(Vec3::ZERO, Vec3::X, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(r.batch().line_vertices.len(), 2);
    }

    #[test]
    fn draw_box_adds_24_verts() {
        let mut r = ImmediateRenderer::new();
        r.draw_box(Vec3::ZERO, Vec3::ONE, [1.0; 4]);
        // 12 edges * 2 verts = 24
        assert_eq!(r.batch().line_vertices.len(), 24);
    }

    #[test]
    fn clear_resets() {
        let mut r = ImmediateRenderer::new();
        r.draw_line(Vec3::ZERO, Vec3::X, [1.0; 4]);
        r.draw_filled_rect(Vec3::ZERO, Vec3::ONE, [1.0; 4]);
        assert!(!r.batch().line_vertices.is_empty());
        assert!(!r.batch().triangle_vertices.is_empty());

        r.clear();
        assert!(r.batch().line_vertices.is_empty());
        assert!(r.batch().triangle_vertices.is_empty());
        assert!(r.batch().triangle_indices.is_empty());
    }
}
