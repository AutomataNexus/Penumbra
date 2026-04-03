//! penumbra-asset -- Asset loading pipeline for Penumbra.

use std::path::Path;

use penumbra_backend::{MeshDescriptor, TextureDescriptor, TextureFormat, TextureUsage, Vertex};
use penumbra_core::{AlphaMode, Material, MaterialId, Rgb, Rgba};
use thiserror::Error;

// ── Errors ──

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image decode error: {0}")]
    ImageDecode(String),
    #[error("GLTF error: {0}")]
    Gltf(String),
    #[error("OBJ parse error: {0}")]
    ObjParse(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

// ── Loaded types ──

#[derive(Debug, Clone)]
pub struct LoadedMesh {
    pub descriptor: MeshDescriptor,
    pub material: Material,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct LoadedScene {
    pub meshes: Vec<LoadedMesh>,
    pub materials: Vec<Material>,
}

#[derive(Debug, Clone)]
pub struct LoadedTexture {
    pub descriptor: TextureDescriptor,
    pub name: String,
}

// ── GLTF loading ──

pub fn load_gltf(path: impl AsRef<Path>) -> Result<LoadedScene, AssetError> {
    let data = std::fs::read(path.as_ref())?;
    load_gltf_bytes(&data)
}

pub fn load_gltf_bytes(data: &[u8]) -> Result<LoadedScene, AssetError> {
    let gltf = gltf::Gltf::from_slice(data).map_err(|e| AssetError::Gltf(e.to_string()))?;
    let buffers = gltf::import_buffers(&gltf.document, None, gltf.blob)
        .map_err(|e| AssetError::Gltf(e.to_string()))?;

    let mut materials = Vec::new();
    for (i, mat) in gltf.document.materials().enumerate() {
        let pbr = mat.pbr_metallic_roughness();
        let base = pbr.base_color_factor();
        let emissive = mat.emissive_factor();
        materials.push(Material {
            id: MaterialId(i as u64),
            albedo: Rgba::new(base[0], base[1], base[2], base[3]),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            emissive: Rgb::new(emissive[0], emissive[1], emissive[2]),
            double_sided: mat.double_sided(),
            alpha_mode: match mat.alpha_mode() {
                gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                gltf::material::AlphaMode::Mask => AlphaMode::Mask {
                    cutoff: mat.alpha_cutoff().unwrap_or(0.5),
                },
                gltf::material::AlphaMode::Blend => AlphaMode::Blend,
            },
            ..Material::default()
        });
    }

    let mut meshes = Vec::new();
    for mesh in gltf.document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .map(|iter| iter.collect())
                .unwrap_or_default();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let tangents: Vec<[f32; 4]> = reader
                .read_tangents()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);

            let vertices: Vec<Vertex> = positions
                .iter()
                .enumerate()
                .map(|(i, pos)| Vertex {
                    position: *pos,
                    normal: normals[i],
                    uv: uvs[i],
                    tangent: tangents[i],
                })
                .collect();

            let indices: Vec<u32> = reader
                .read_indices()
                .map(|iter| iter.into_u32().collect())
                .unwrap_or_default();

            let mat_index = primitive.material().index().unwrap_or(0);
            let material = materials.get(mat_index).cloned().unwrap_or_default();

            meshes.push(LoadedMesh {
                descriptor: MeshDescriptor {
                    vertices,
                    indices,
                    label: Some(mesh.name().unwrap_or("unnamed").to_string()),
                },
                material,
                name: mesh.name().unwrap_or("unnamed").to_string(),
            });
        }
    }

    Ok(LoadedScene { meshes, materials })
}

// ── Image loading ──

pub fn load_image(path: impl AsRef<Path>) -> Result<LoadedTexture, AssetError> {
    let data = std::fs::read(path.as_ref())?;
    let name = path
        .as_ref()
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());
    let mut tex = load_image_bytes(&data)?;
    tex.name = name;
    Ok(tex)
}

pub fn load_image_bytes(data: &[u8]) -> Result<LoadedTexture, AssetError> {
    let img = image::load_from_memory(data)
        .map_err(|e| AssetError::ImageDecode(e.to_string()))?
        .to_rgba8();
    let (w, h) = img.dimensions();
    Ok(LoadedTexture {
        descriptor: TextureDescriptor {
            width: w,
            height: h,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
            data: Some(img.into_raw()),
            label: Some("loaded_image".to_string()),
            mip_levels: 1,
        },
        name: "image".to_string(),
    })
}

// ── OBJ loading ──

pub fn load_obj(path: impl AsRef<Path>) -> Result<LoadedMesh, AssetError> {
    let text = std::fs::read_to_string(path.as_ref())?;
    parse_obj(&text)
}

pub fn parse_obj(text: &str) -> Result<LoadedMesh, AssetError> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    // map from (pos_idx, uv_idx, norm_idx) -> vertex index
    let mut vertex_map: std::collections::HashMap<(usize, usize, usize), u32> =
        std::collections::HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "v" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                positions.push([x, y, z]);
            }
            "vn" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                normals.push([x, y, z]);
            }
            "vt" if parts.len() >= 3 => {
                let u: f32 = parts[1].parse().unwrap_or(0.0);
                let v: f32 = parts[2].parse().unwrap_or(0.0);
                uvs.push([u, v]);
            }
            "f" if parts.len() >= 4 => {
                // Triangulate face (fan triangulation)
                let face_verts: Vec<(usize, usize, usize)> =
                    parts[1..].iter().map(|s| parse_face_vertex(s)).collect();
                for i in 1..face_verts.len() - 1 {
                    for &fi in &[0, i, i + 1] {
                        let key = face_verts[fi];
                        let idx = if let Some(&existing) = vertex_map.get(&key) {
                            existing
                        } else {
                            let pos = if key.0 > 0 && key.0 <= positions.len() {
                                positions[key.0 - 1]
                            } else {
                                [0.0, 0.0, 0.0]
                            };
                            let uv = if key.1 > 0 && key.1 <= uvs.len() {
                                uvs[key.1 - 1]
                            } else {
                                [0.0, 0.0]
                            };
                            let normal = if key.2 > 0 && key.2 <= normals.len() {
                                normals[key.2 - 1]
                            } else {
                                [0.0, 1.0, 0.0]
                            };
                            let new_idx = vertices.len() as u32;
                            vertices.push(Vertex {
                                position: pos,
                                normal,
                                uv,
                                tangent: [1.0, 0.0, 0.0, 1.0],
                            });
                            vertex_map.insert(key, new_idx);
                            new_idx
                        };
                        indices.push(idx);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(LoadedMesh {
        descriptor: MeshDescriptor {
            vertices,
            indices,
            label: Some("obj_mesh".to_string()),
        },
        material: Material::default(),
        name: "obj_mesh".to_string(),
    })
}

fn parse_face_vertex(s: &str) -> (usize, usize, usize) {
    let parts: Vec<&str> = s.split('/').collect();
    let pos = parts
        .first()
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(0);
    let uv = parts
        .get(1)
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(0);
    let norm = parts
        .get(2)
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(0);
    (pos, uv, norm)
}

// ── Primitive generators ──

/// Generate a cube mesh with 24 vertices (4 per face) and 36 indices.
pub fn cube_mesh() -> MeshDescriptor {
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    // Each face has 4 vertices, 6 faces = 24 vertices
    type CubeFace = ([f32; 3], [[f32; 3]; 4], [[f32; 2]; 4]);
    let faces: [CubeFace; 6] = [
        // +Z face
        (
            [0.0, 0.0, 1.0],
            [
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        // -Z face
        (
            [0.0, 0.0, -1.0],
            [
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
            ],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        // +X face
        (
            [1.0, 0.0, 0.0],
            [
                [0.5, -0.5, 0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
            ],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        // -X face
        (
            [-1.0, 0.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, 0.5, -0.5],
            ],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        // +Y face
        (
            [0.0, 1.0, 0.0],
            [
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
            ],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        // -Y face
        (
            [0.0, -1.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
    ];

    for (normal, positions, uvs) in &faces {
        let base = vertices.len() as u32;
        for i in 0..4 {
            vertices.push(Vertex {
                position: positions[i],
                normal: *normal,
                uv: uvs[i],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    MeshDescriptor {
        vertices,
        indices,
        label: Some("cube".to_string()),
    }
}

/// Generate a UV sphere with the given number of segments and rings.
pub fn sphere_mesh(segments: u32, rings: u32) -> MeshDescriptor {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let theta = std::f32::consts::PI * ring as f32 / rings as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for seg in 0..=segments {
            let phi = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;
            let u = seg as f32 / segments as f32;
            let v = ring as f32 / rings as f32;

            vertices.push(Vertex {
                position: [x * 0.5, y * 0.5, z * 0.5],
                normal: [x, y, z],
                uv: [u, v],
                tangent: [-sin_phi, 0.0, cos_phi, 1.0],
            });
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let current = ring * (segments + 1) + seg;
            let next = current + segments + 1;
            indices.extend_from_slice(&[current, next, current + 1]);
            indices.extend_from_slice(&[current + 1, next, next + 1]);
        }
    }

    MeshDescriptor {
        vertices,
        indices,
        label: Some("sphere".to_string()),
    }
}

/// Generate a flat plane mesh with the given number of subdivisions per axis.
pub fn plane_mesh(subdivisions: u32) -> MeshDescriptor {
    let verts_per_side = subdivisions + 1;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for z in 0..verts_per_side {
        for x in 0..verts_per_side {
            let fx = x as f32 / subdivisions as f32 - 0.5;
            let fz = z as f32 / subdivisions as f32 - 0.5;
            vertices.push(Vertex {
                position: [fx, 0.0, fz],
                normal: [0.0, 1.0, 0.0],
                uv: [
                    x as f32 / subdivisions as f32,
                    z as f32 / subdivisions as f32,
                ],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
    }

    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let tl = z * verts_per_side + x;
            let tr = tl + 1;
            let bl = tl + verts_per_side;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    MeshDescriptor {
        vertices,
        indices,
        label: Some("plane".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_has_24_vertices_36_indices() {
        let mesh = cube_mesh();
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn sphere_vertex_count() {
        let mesh = sphere_mesh(16, 8);
        let expected_verts = (16 + 1) * (8 + 1);
        assert_eq!(mesh.vertices.len(), expected_verts as usize);
    }

    #[test]
    fn plane_vertex_count() {
        let mesh = plane_mesh(4);
        let expected = (4 + 1) * (4 + 1);
        assert_eq!(mesh.vertices.len(), expected as usize);
    }

    #[test]
    fn obj_parser_cube() {
        let obj = r#"
# Simple cube
v -0.5 -0.5  0.5
v  0.5 -0.5  0.5
v  0.5  0.5  0.5
v -0.5  0.5  0.5
v -0.5 -0.5 -0.5
v  0.5 -0.5 -0.5
v  0.5  0.5 -0.5
v -0.5  0.5 -0.5

f 1 2 3 4
f 5 6 7 8
f 1 5 6 2
f 3 4 8 7
f 1 4 8 5
f 2 6 7 3
"#;
        let mesh = parse_obj(obj).unwrap();
        // 6 faces, each quad = 2 triangles = 6 indices
        assert_eq!(mesh.descriptor.indices.len(), 36);
        // Vertices should be <= 8*3 depending on dedup, but at least 8
        assert!(mesh.descriptor.vertices.len() >= 8);
    }
}
