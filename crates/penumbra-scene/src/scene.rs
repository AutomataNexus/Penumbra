use glam::Mat4;
use penumbra_backend::{Aabb, MeshId};
use penumbra_core::MaterialId;
use slotmap::SlotMap;

use crate::light::Light;
use crate::node::{Renderable, SceneNode};
use crate::transform::Transform;

slotmap::new_key_type! {
    pub struct NodeId;
}

pub struct Scene {
    nodes: SlotMap<NodeId, SceneNode>,
    root: NodeId,
}

impl Scene {
    pub fn new() -> Self {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert(SceneNode::default());
        Self { nodes, root }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn add_empty(&mut self) -> NodeId {
        let id = self.nodes.insert(SceneNode::default());
        self.nodes[id].parent = Some(self.root);
        self.nodes[self.root].children.push(id);
        id
    }

    pub fn add_mesh(&mut self, mesh: MeshId, material: MaterialId) -> NodeId {
        let node = SceneNode {
            renderable: Some(Renderable::Mesh { mesh, material }),
            parent: Some(self.root),
            ..Default::default()
        };
        let id = self.nodes.insert(node);
        self.nodes[self.root].children.push(id);
        id
    }

    pub fn add_light(&mut self, light: Light) -> NodeId {
        let node = SceneNode {
            renderable: Some(Renderable::Light { light }),
            parent: Some(self.root),
            ..Default::default()
        };
        let id = self.nodes.insert(node);
        self.nodes[self.root].children.push(id);
        id
    }

    pub fn set_transform(&mut self, id: NodeId, transform: Transform) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.transform = transform;
        }
    }

    pub fn set_parent(&mut self, child: NodeId, parent: NodeId) {
        // Remove from old parent
        if let Some(old_parent) = self.nodes.get(child).and_then(|n| n.parent) {
            if let Some(parent_node) = self.nodes.get_mut(old_parent) {
                parent_node.children.retain(|&c| c != child);
            }
        }
        // Set new parent
        if let Some(node) = self.nodes.get_mut(child) {
            node.parent = Some(parent);
        }
        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.children.push(child);
        }
    }

    pub fn set_visible(&mut self, id: NodeId, visible: bool) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.visible = visible;
        }
    }

    pub fn set_aabb(&mut self, id: NodeId, aabb: Aabb) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.aabb = Some(aabb);
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id)
    }

    pub fn remove_node(&mut self, id: NodeId) {
        if id == self.root {
            return;
        }
        // Remove from parent
        if let Some(parent_id) = self.nodes.get(id).and_then(|n| n.parent) {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.retain(|&c| c != id);
            }
        }
        // Collect children to reparent to root
        let children: Vec<NodeId> = self
            .nodes
            .get(id)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        for child in children {
            self.set_parent(child, self.root);
        }
        self.nodes.remove(id);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn update_transforms(&mut self) {
        self.propagate_transform(self.root, Mat4::IDENTITY);
    }

    fn propagate_transform(&mut self, id: NodeId, parent_world: Mat4) {
        let local = self.nodes[id].transform.to_matrix();
        let world = parent_world * local;
        self.nodes[id].world_transform = world;

        let children: Vec<NodeId> = self.nodes[id].children.clone();
        for child in children {
            self.propagate_transform(child, world);
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn new_scene_has_root() {
        let scene = Scene::new();
        assert_eq!(scene.node_count(), 1);
        assert!(scene.get_node(scene.root()).is_some());
    }

    #[test]
    fn add_and_remove_nodes() {
        let mut scene = Scene::new();
        let n1 = scene.add_empty();
        let n2 = scene.add_empty();
        assert_eq!(scene.node_count(), 3);
        scene.remove_node(n1);
        assert_eq!(scene.node_count(), 2);
        assert!(scene.get_node(n1).is_none());
        assert!(scene.get_node(n2).is_some());
    }

    #[test]
    fn parent_child_hierarchy() {
        let mut scene = Scene::new();
        let parent = scene.add_empty();
        let child = scene.add_empty();
        scene.set_parent(child, parent);
        assert_eq!(scene.get_node(child).unwrap().parent, Some(parent));
        assert!(scene.get_node(parent).unwrap().children.contains(&child));
    }

    #[test]
    fn transform_propagation() {
        let mut scene = Scene::new();
        let parent = scene.add_empty();
        let child = scene.add_empty();
        scene.set_parent(child, parent);
        scene.set_transform(
            parent,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        );
        scene.set_transform(
            child,
            Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
        );
        scene.update_transforms();
        let child_world = scene.get_node(child).unwrap().world_transform;
        let pos = child_world.col(3).truncate();
        assert!((pos.x - 10.0).abs() < 0.001);
        assert!((pos.y - 5.0).abs() < 0.001);
    }

    #[test]
    fn deep_hierarchy_propagation() {
        let mut scene = Scene::new();
        let mut prev = scene.root();
        for _i in 0..10 {
            let node = scene.add_empty();
            scene.set_parent(node, prev);
            scene.set_transform(
                node,
                Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            );
            prev = node;
        }
        scene.update_transforms();
        let leaf = prev;
        let leaf_world = scene.get_node(leaf).unwrap().world_transform;
        let pos = leaf_world.col(3).truncate();
        assert!((pos.x - 10.0).abs() < 0.001);
    }

    #[test]
    fn add_mesh_node() {
        let mut scene = Scene::new();
        let id = scene.add_mesh(MeshId(1), MaterialId(1));
        let node = scene.get_node(id).unwrap();
        assert!(matches!(node.renderable, Some(Renderable::Mesh { .. })));
    }
}
