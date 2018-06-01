use cgmath::{Matrix4, SquareMatrix};

use scene::camera::Camera;
use scene::light::{Light, LightBuilder};

pub struct TreeNode<T> {
    data: Option<T>,
    transform: Matrix4<f32>,
    cached_transform: Option<Matrix4<f32>>,
    children: Vec<TreeNode<T>>
}

impl<T> TreeNode<T> {
    fn new(data: Option<T>) -> Self {
        TreeNode {
            data: data,
            transform: Matrix4::identity(),
            cached_transform: None,
            children: Vec::new()
        }
    }

    fn with_transform(data: Option<T>, transform: Matrix4<f32>) -> Self {
        TreeNode {
            data: data,
            transform: transform,
            cached_transform: None,
            children: Vec::new()
        }
    }

    pub fn insert(&mut self, data: T) -> &TreeNode<T> {
        let child = TreeNode::new(Some(data));

        self.children.push(child);

        self.children.last().unwrap()
    }

    pub fn insert_with_transform(&mut self, data: T, transform: Matrix4<f32>)
    -> &TreeNode<T> {
        let child = TreeNode::with_transform(Some(data), transform);

        self.children.push(child);

        self.children.last().unwrap()
    }
}

pub struct TreeGraph<T> {
    camera: Camera,
    light: Light,
    root: TreeNode<T>
}

impl<T> TreeGraph<T> {
    pub fn new() -> TreeGraph<T> {
        TreeGraph {
            camera: Camera::new([0., 0., 0.]),
            light: LightBuilder::new().build(),
            root: TreeNode::new(None)
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn light(&self) -> &Light {
        &self.light
    }

    pub fn set_light(&mut self, light: Light) {
        self.light = light;
    }

    pub fn root(&self) -> &TreeNode<T> {
        &self.root
    }

    pub fn insert<O>(&mut self, object: O) -> &TreeNode<T> where O: Into<T> {
        self.root.insert(object.into())
    }

    pub fn insert_with_transform<O>(&mut self, object: O, transform: Matrix4<f32>)
    -> &TreeNode<T> where O: Into<T> {
        self.root.insert_with_transform(object.into(), transform)
    }

    pub fn foreach<F>(&mut self, mut f: F)
    where F: FnMut(&T, Matrix4<f32>) {
        TreeGraph::visit(&mut self.root, &mut f, Matrix4::identity());
    }

    fn visit<F>(node: &mut TreeNode<T>, f: &mut F, parent_transform: Matrix4<f32>)
    where F: FnMut(&T, Matrix4<f32>) {
        if node.cached_transform.is_none() {
            node.cached_transform = Some(parent_transform * node.transform);
        }

        let transform = node.cached_transform.unwrap();

        for child in &mut node.children {
            TreeGraph::visit(child, f, transform);
        }

        match &node.data {
            Some(d) => f(d, transform),
            _ => ()
        }
    }

    pub fn clear(&mut self) {
        self.root = TreeNode::new(None);
    }
}
