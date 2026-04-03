use glam::Mat4;
use penumbra_backend::{Aabb, MeshId};
use penumbra_core::MaterialId;

use crate::light::Light;
use crate::scene::NodeId;
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub enum Renderable {
    Mesh { mesh: MeshId, material: MaterialId },
    Light { light: Light },
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub transform: Transform,
    pub world_transform: Mat4,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub renderable: Option<Renderable>,
    pub visible: bool,
    pub name: Option<String>,
    pub aabb: Option<Aabb>,
}

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            world_transform: Mat4::IDENTITY,
            parent: None,
            children: Vec::new(),
            renderable: None,
            visible: true,
            name: None,
            aabb: None,
        }
    }
}
