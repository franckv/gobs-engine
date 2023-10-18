use crate::model::Transform;
use crate::scene::camera::Camera;
use crate::scene::light::{Light, LightBuilder};

pub struct SceneNode<D> {
    data: Option<D>,
    children: Option<Vec<SceneNode<D>>>,
    transform: Transform,
    model_transform: Transform,
    cached_transform: Option<Transform>,
    cached_normal: Option<Transform>,
}

impl<D> SceneNode<D> {
    pub fn insert(&mut self, child: SceneNode<D>) {
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

    pub fn data(&self) -> &Option<D> {
        &self.data
    }

    pub fn transform_mut(&mut self) -> &Transform {
        &mut self.transform
    }
}

impl<D> Default for SceneNode<D> {
    fn default() -> Self {
        SceneNode {
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
    children: Option<Vec<SceneNode<D>>>,
    transform: Option<Transform>,
    model_transform: Option<Transform>,
}

impl<D> NodeBuilder<D> {
    pub fn data(mut self, data: D) -> Self {
        self.data = Some(data);

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

    pub fn child(mut self, child: SceneNode<D>) -> Self {
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

    pub fn build(self) -> SceneNode<D> {
        let transform = match self.transform {
            Some(transform) => transform,
            None => Transform::new(),
        };

        let model_transform = match self.model_transform {
            Some(model_transform) => model_transform,
            None => Transform::new(),
        };

        SceneNode {
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

pub struct SceneGraph<D> {
    camera: Camera,
    light: Light,
    root: SceneNode<D>,
    dirty: bool,
}

impl<D> SceneGraph<D> {
    pub fn new() -> SceneGraph<D> {
        SceneGraph {
            camera: Camera::ortho(1., 1.),
            light: LightBuilder::new().build(),
            root: SceneNode::default(),
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

    pub fn root(&self) -> &SceneNode<D> {
        &self.root
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true
    }

    pub fn new_node() -> NodeBuilder<D> {
        NodeBuilder::default()
    }

    pub fn insert(&mut self, child: SceneNode<D>) {
        self.dirty = true;
        self.root.insert(child);
    }

    pub fn foreach<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut SceneNode<D>, &Transform),
    {
        SceneGraph::visit(&mut self.root, &mut f, &Transform::new());
        self.dirty = false;
    }

    fn visit<F>(node: &mut SceneNode<D>, f: &mut F, parent_transform: &Transform)
    where
        F: FnMut(&mut SceneNode<D>, &Transform),
    {
        let transform = {
            if node.cached_transform.is_none() {
                node.cached_transform = Some(
                    node.model_transform
                        .clone()
                        .transform(&node.transform)
                        .transform(&parent_transform),
                );

                node.cached_normal =
                    Some(node.cached_transform.as_ref().unwrap().normal_transform());
            }

            let transform = node.cached_transform.as_ref().unwrap().clone();

            if let Some(ref mut children) = node.children {
                for child in children {
                    SceneGraph::visit(child, f, &transform);
                }
            }

            transform
        };

        f(node, &transform);

        /*
        match &node.data {
            Some(d) => f(node, transform),
            _ => ()
        }
         */
    }

    pub fn clear(&mut self) {
        self.dirty = true;
        self.root = SceneNode::default();
    }
}
