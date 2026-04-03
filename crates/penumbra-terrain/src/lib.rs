//! penumbra-terrain -- Tile-based terrain streaming for Penumbra.

use penumbra_backend::{MeshDescriptor, Vertex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

// ── Errors ──

#[derive(Debug, Error)]
pub enum TerrainError {
    #[error("Tile not found: {0:?}")]
    TileNotFound(TileCoord),
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Tile coord ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
}

impl TileCoord {
    pub fn new(x: u32, y: u32, zoom: u32) -> Self {
        Self { x, y, zoom }
    }

    /// Parent tile at one zoom level up.
    pub fn parent(&self) -> Option<TileCoord> {
        if self.zoom == 0 {
            return None;
        }
        Some(TileCoord {
            x: self.x / 2,
            y: self.y / 2,
            zoom: self.zoom - 1,
        })
    }

    /// The four children at one zoom level down.
    pub fn children(&self) -> [TileCoord; 4] {
        let x2 = self.x * 2;
        let y2 = self.y * 2;
        let z1 = self.zoom + 1;
        [
            TileCoord::new(x2, y2, z1),
            TileCoord::new(x2 + 1, y2, z1),
            TileCoord::new(x2, y2 + 1, z1),
            TileCoord::new(x2 + 1, y2 + 1, z1),
        ]
    }
}

// ── Tile data ──

#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TerrainData {
    pub heights: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum TileData {
    Image(ImageData),
    Terrain(TerrainData),
}

// ── Tile source ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileFormat {
    Png,
    Jpeg,
    TerrainRgb,
}

pub trait TileSource: Send + Sync {
    fn tile_url(&self, coord: TileCoord) -> String;
    fn format(&self) -> TileFormat;
}

/// XYZ tile source using a URL template with {x}, {y}, {z} placeholders.
#[derive(Debug, Clone)]
pub struct XyzTileSource {
    pub url_template: String,
    pub tile_format: TileFormat,
}

impl XyzTileSource {
    pub fn new(url_template: &str, format: TileFormat) -> Self {
        Self {
            url_template: url_template.to_string(),
            tile_format: format,
        }
    }
}

impl TileSource for XyzTileSource {
    fn tile_url(&self, coord: TileCoord) -> String {
        self.url_template
            .replace("{x}", &coord.x.to_string())
            .replace("{y}", &coord.y.to_string())
            .replace("{z}", &coord.zoom.to_string())
    }

    fn format(&self) -> TileFormat {
        self.tile_format
    }
}

// ── Tile cache ──

pub struct TileCache {
    capacity: usize,
    tiles: HashMap<TileCoord, TileData>,
    order: VecDeque<TileCoord>,
}

impl TileCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            tiles: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn get(&self, coord: &TileCoord) -> Option<&TileData> {
        self.tiles.get(coord)
    }

    pub fn insert(&mut self, coord: TileCoord, data: TileData) {
        if self.tiles.contains_key(&coord) {
            // Move to back (most recently used)
            self.order.retain(|c| c != &coord);
            self.order.push_back(coord);
            self.tiles.insert(coord, data);
            return;
        }
        // Evict if at capacity
        while self.tiles.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.tiles.remove(&oldest);
            } else {
                break;
            }
        }
        self.tiles.insert(coord, data);
        self.order.push_back(coord);
    }

    pub fn contains(&self, coord: &TileCoord) -> bool {
        self.tiles.contains_key(coord)
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

// ── Terrain config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub tile_size: u32,
    pub max_zoom: u32,
    pub height_scale: f32,
    pub mesh_resolution: u32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            tile_size: 256,
            max_zoom: 15,
            height_scale: 1.0,
            mesh_resolution: 32,
        }
    }
}

// ── Terrain mesh ──

#[derive(Debug, Clone)]
pub struct TerrainMesh {
    pub descriptor: MeshDescriptor,
    pub coord: TileCoord,
}

/// Decode Mapbox Terrain-RGB encoded elevation.
/// height = -10000 + ((R * 256 * 256 + G * 256 + B) * 0.1)
pub fn decode_terrain_rgb(r: u8, g: u8, b: u8) -> f32 {
    -10000.0 + (r as f32 * 256.0 * 256.0 + g as f32 * 256.0 + b as f32) * 0.1
}

/// Generate a terrain tile mesh from height data.
pub fn generate_tile_mesh(
    coord: TileCoord,
    heights: &[f32],
    resolution: u32,
    tile_size: f32,
    height_scale: f32,
) -> TerrainMesh {
    let verts_per_side = resolution + 1;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let step = tile_size / resolution as f32;

    for z in 0..verts_per_side {
        for x in 0..verts_per_side {
            let idx = (z * verts_per_side + x) as usize;
            let h = heights.get(idx).copied().unwrap_or(0.0) * height_scale;
            let px = x as f32 * step;
            let pz = z as f32 * step;
            let u = x as f32 / resolution as f32;
            let v = z as f32 / resolution as f32;

            vertices.push(Vertex {
                position: [px, h, pz],
                normal: [0.0, 1.0, 0.0],
                uv: [u, v],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
    }

    // Compute face normals (simple)
    for z in 0..resolution {
        for x in 0..resolution {
            let tl = z * verts_per_side + x;
            let tr = tl + 1;
            let bl = tl + verts_per_side;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    TerrainMesh {
        descriptor: MeshDescriptor {
            vertices,
            indices,
            label: Some(format!("terrain_{}_{}_z{}", coord.x, coord.y, coord.zoom)),
        },
        coord,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_parent_children() {
        let tile = TileCoord::new(4, 6, 3);
        let parent = tile.parent().unwrap();
        assert_eq!(parent, TileCoord::new(2, 3, 2));

        let children = tile.children();
        assert_eq!(children[0], TileCoord::new(8, 12, 4));
        assert_eq!(children[1], TileCoord::new(9, 12, 4));
        assert_eq!(children[2], TileCoord::new(8, 13, 4));
        assert_eq!(children[3], TileCoord::new(9, 13, 4));
    }

    #[test]
    fn tile_root_has_no_parent() {
        let tile = TileCoord::new(0, 0, 0);
        assert!(tile.parent().is_none());
    }

    #[test]
    fn cache_lru_eviction() {
        let mut cache = TileCache::new(2);
        let t1 = TileCoord::new(0, 0, 0);
        let t2 = TileCoord::new(1, 0, 0);
        let t3 = TileCoord::new(2, 0, 0);

        cache.insert(
            t1,
            TileData::Terrain(TerrainData {
                heights: vec![0.0],
                width: 1,
                height: 1,
            }),
        );
        cache.insert(
            t2,
            TileData::Terrain(TerrainData {
                heights: vec![1.0],
                width: 1,
                height: 1,
            }),
        );
        assert_eq!(cache.len(), 2);

        // Insert t3, should evict t1
        cache.insert(
            t3,
            TileData::Terrain(TerrainData {
                heights: vec![2.0],
                width: 1,
                height: 1,
            }),
        );
        assert_eq!(cache.len(), 2);
        assert!(!cache.contains(&t1));
        assert!(cache.contains(&t2));
        assert!(cache.contains(&t3));
    }

    #[test]
    fn url_template() {
        let source =
            XyzTileSource::new("https://tile.example.com/{z}/{x}/{y}.png", TileFormat::Png);
        let url = source.tile_url(TileCoord::new(3, 5, 10));
        assert_eq!(url, "https://tile.example.com/10/3/5.png");
    }

    #[test]
    fn terrain_rgb_decode() {
        // Known value: RGB(1, 134, 160) should decode to specific height
        let h = decode_terrain_rgb(1, 134, 160);
        let expected = -10000.0 + (1.0 * 256.0 * 256.0 + 134.0 * 256.0 + 160.0) * 0.1;
        assert!((h - expected).abs() < 0.01);
    }

    #[test]
    fn mesh_generation() {
        let resolution = 4;
        let verts_per_side = resolution + 1;
        let heights = vec![0.0; (verts_per_side * verts_per_side) as usize];
        let mesh = generate_tile_mesh(TileCoord::new(0, 0, 0), &heights, resolution, 1.0, 1.0);
        assert_eq!(
            mesh.descriptor.vertices.len(),
            (verts_per_side * verts_per_side) as usize
        );
        assert_eq!(
            mesh.descriptor.indices.len(),
            (resolution * resolution * 6) as usize
        );
    }
}
