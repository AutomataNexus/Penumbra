use penumbra_backend::MeshId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodLevel {
    pub mesh: MeshId,
    pub max_screen_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodMesh {
    pub levels: Vec<LodLevel>,
}

impl LodMesh {
    pub fn new(levels: Vec<LodLevel>) -> Self {
        Self { levels }
    }

    /// Select the appropriate LOD level for a given screen-space size.
    pub fn select_level(&self, screen_size: f32) -> Option<&LodLevel> {
        self.levels
            .iter()
            .find(|level| screen_size <= level.max_screen_size)
            .or(self.levels.last())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_correct_lod() {
        let lod = LodMesh::new(vec![
            LodLevel {
                mesh: MeshId(0),
                max_screen_size: 50.0,
            },
            LodLevel {
                mesh: MeshId(1),
                max_screen_size: 200.0,
            },
            LodLevel {
                mesh: MeshId(2),
                max_screen_size: 1000.0,
            },
        ]);

        assert_eq!(lod.select_level(10.0).unwrap().mesh, MeshId(0));
        assert_eq!(lod.select_level(100.0).unwrap().mesh, MeshId(1));
        assert_eq!(lod.select_level(500.0).unwrap().mesh, MeshId(2));
        assert_eq!(lod.select_level(5000.0).unwrap().mesh, MeshId(2));
    }
}
