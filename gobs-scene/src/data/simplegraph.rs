use std::sync::{Arc, RwLock};

use cgmath::{Matrix4, SquareMatrix};

use scene::camera::Camera;
use scene::light::{Light, LightBuilder};

type NodeRef<T> = Arc<RwLock<Node<T>>>;

pub struct SimpleNode<T> {
    node: NodeRef<T>,
}

impl<T> SimpleNode<T> {
    fn new(node: NodeRef<T>) -> Self {
        SimpleNode {
            node: node
        }
    }

    pub fn insert(&mut self, data: T) -> SimpleNode<T> {
        let child = Node::new(Some(data));

        self.node.write().unwrap().insert(child.clone());

        SimpleNode {
            node: child
        }
    }

    pub fn insert_with_transform(&mut self, data: T, transform: Matrix4<f32>)
    -> SimpleNode<T> {
        let child = Node::with_transform(Some(data), transform);

        self.node.write().unwrap().insert(child.clone());

        SimpleNode {
            node: child
        }
    }
}

struct Node<T> {
    data: Option<T>,
    transform: Matrix4<f32>,
    cached_transform: Option<Matrix4<f32>>,
    children: Vec<NodeRef<T>>
}

impl<T> Node<T> {
    fn new(data: Option<T>) -> NodeRef<T> {
        Arc::new(RwLock::new(Node {
            data: data,
            transform: Matrix4::identity(),
            cached_transform: None,
            children: Vec::new()
        }))
    }

    fn with_transform(data: Option<T>, transform: Matrix4<f32>) -> NodeRef<T> {
        Arc::new(RwLock::new(Node {
            data: data,
            transform: transform,
            cached_transform: None,
            children: Vec::new()
        }))
    }

    fn insert(&mut self, child: NodeRef<T>) {
        self.children.push(child);
    }
}

pub struct SimpleGraph<T> {
    camera: Camera,
    light: Light,
    root: SimpleNode<T>
}

impl<T> SimpleGraph<T> {
    pub fn new() -> SimpleGraph<T> {
        SimpleGraph {
            camera: Camera::new([0., 0., 0.]),
            light: LightBuilder::new().build(),
            root: SimpleNode::new(Node::new(None))
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

    pub fn root(&self) -> &SimpleNode<T> {
        &self.root
    }

    pub fn insert<O>(&mut self, object: O) -> SimpleNode<T> where O: Into<T> {
        self.root.insert(object.into())
    }

    pub fn insert_with_transform<O>(&mut self, object: O, transform: Matrix4<f32>)
    -> SimpleNode<T> where O: Into<T> {
        self.root.insert_with_transform(object.into(), transform)
    }

    pub fn foreach<F>(&mut self, mut f: F)
    where F: FnMut(&T, Matrix4<f32>) {
        SimpleGraph::visit(&mut self.root.node, &mut f, Matrix4::identity());
    }

    fn visit<F>(node: &mut NodeRef<T>, f: &mut F, parent_transform: Matrix4<f32>)
    where F: FnMut(&T, Matrix4<f32>) {
        let mut node = node.write().unwrap();

        if node.cached_transform.is_none() {
            node.cached_transform = Some(parent_transform * node.transform);
        }

        let transform = node.cached_transform.unwrap();

        for mut child in &mut node.children {
            SimpleGraph::visit(&mut child, f, transform);
        }

        match &node.data {
            Some(d) => f(d, transform),
            _ => ()
        }
    }

    pub fn clear(&mut self) {
        self.root = SimpleNode::new(Node::new(None));
    }
}
