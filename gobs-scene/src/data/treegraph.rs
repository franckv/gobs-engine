use crate::model::Transform;
use crate::scene::{
    camera::Camera,
    light::{Light, LightBuilder},
};

pub struct TreeNode<D> {
    data: Option<D>,
    children: Option<Vec<TreeNode<D>>>,
    transform: Transform,
    model_transform: Transform,
    cached_transform: Option<Transform>,
    cached_normal: Option<Transform>,
}

impl<D> TreeNode<D> {
    pub fn insert(&mut self, child: TreeNode<D>) {
        match self.children {
            Some(ref mut vec) => {
                vec.push(child);
            }
            None => {
                let mut vec = Vec::new();
                vec.push(child);
                self.children = Some(vec);
            }
        }
    }
}

impl<D> Default for TreeNode<D> {
    fn default() -> Self {
        TreeNode {
            data: None,
            children: None,
            transform: Transform::new(),
            model_transform: Transform::new(),
            cached_transform: None,
            cached_normal: None,
        }
    }
}

pub struct NodeBuilder<D> {
    data: Option<D>,
    children: Option<Vec<TreeNode<D>>>,
    transform: Option<Transform>,
    model_transform: Option<Transform>,
}

impl<D> NodeBuilder<D> {
    pub fn data<O: Into<D>>(mut self, data: O) -> Self {
        self.data = Some(data.into());

        self
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);

        self
    }

    pub fn model_transform(mut self, model_transform: Transform) -> Self {
        self.model_transform = Some(model_transform);

        self
    }

    pub fn child(mut self, child: TreeNode<D>) -> Self {
        match self.children {
            Some(ref mut vec) => {
                vec.push(child);
            }
            None => {
                let mut vec = Vec::new();
                vec.push(child);
                self.children = Some(vec);
            }
        }

        self
    }

    pub fn build(self) -> TreeNode<D> {
        let transform = match self.transform {
            Some(transform) => transform,
            None => Transform::new(),
        };

        let model_transform = match self.model_transform {
            Some(model_transform) => model_transform,
            None => Transform::new(),
        };

        TreeNode {
            data: self.data,
            children: self.children,
            transform: transform,
            model_transform: model_transform,
            cached_transform: None,
            cached_normal: None,
        }
    }
}

impl<D> Default for NodeBuilder<D> {
    fn default() -> Self {
        NodeBuilder {
            data: None,
            children: None,
            transform: None,
            model_transform: None,
        }
    }
}

pub struct TreeGraph<D> {
    camera: Camera,
    light: Light,
    root: TreeNode<D>,
    dirty: bool,
}

impl<D> TreeGraph<D> {
    pub fn new() -> TreeGraph<D> {
        TreeGraph {
            camera: Camera::ortho(1., 1.),
            light: LightBuilder::new().build(),
            root: TreeNode::default(),
            dirty: true,
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        self.dirty = true;
        &mut self.camera
    }

    pub fn light(&self) -> &Light {
        &self.light
    }

    pub fn set_light(&mut self, light: Light) {
        self.dirty = true;
        self.light = light;
    }

    pub fn root(&self) -> &TreeNode<D> {
        &self.root
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn new_node() -> NodeBuilder<D> {
        NodeBuilder::default()
    }

    pub fn insert(&mut self, child: TreeNode<D>) {
        self.dirty = true;
        self.root.insert(child);
    }

    pub fn foreach<F>(&mut self, mut f: F)
    where
        F: FnMut(&D, &Transform),
    {
        TreeGraph::visit(&mut self.root, &mut f, &Transform::new());
        self.dirty = false;
    }

    fn visit<F>(node: &mut TreeNode<D>, f: &mut F, parent_transform: &Transform)
    where
        F: FnMut(&D, &Transform),
    {
        if node.cached_transform.is_none() {
            node.cached_transform = Some(
                node.model_transform
                    .clone()
                    .transform(&node.transform)
                    .transform(&parent_transform),
            );

            node.cached_normal = Some(node.cached_transform.as_ref().unwrap().normal_transform());
        }

        let transform = node.cached_transform.as_ref().unwrap();

        if let Some(ref mut children) = node.children {
            for child in children {
                TreeGraph::visit(child, f, transform);
            }
        }

        match &node.data {
            Some(d) => f(d, transform),
            _ => (),
        }
    }

    pub fn clear(&mut self) {
        self.dirty = true;
        self.root = TreeNode::default();
    }
}
