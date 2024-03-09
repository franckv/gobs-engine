use std::sync::Arc;

use gobs_core::{
    entity::{camera::Camera, light::Light},
    Transform,
};
use gobs_render::{
    context::Context,
    pass::RenderPass,
    renderable::{RenderBatch, Renderable},
};

use crate::graph::scenegraph::{NodeId, NodeValue, SceneGraph};

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: NodeId,
    pub light: Light,
}

impl Scene {
    pub fn new(camera: Camera, light: Light) -> Self {
        let mut graph = SceneGraph::new();
        let camera = graph
            .insert(graph.root, NodeValue::Camera(camera), Transform::IDENTITY)
            .expect("Cannot insert camera");

        Scene {
            graph,
            camera,
            light,
        }
    }

    pub fn update(&mut self, _ctx: &Context, _delta: f32) {}

    pub fn camera(&self) -> &Camera {
        if let NodeValue::Camera(camera) =
            &self.graph.get(self.camera).expect("Camera not found").value
        {
            &camera
        } else {
            unreachable!()
        }
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        if let NodeValue::Camera(ref mut camera) = &mut self
            .graph
            .get_mut(self.camera)
            .expect("Camera not found")
            .value
        {
            camera
        } else {
            unreachable!("{:?}", self.camera)
        }
    }
}

impl Renderable for Scene {
    fn resize(&mut self, width: u32, height: u32) {
        self.camera_mut().resize(width, height);
    }

    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch) {
        self.graph.visit(self.graph.root, &mut |&transform, model| {
            if let NodeValue::Model(model) = model {
                batch.add_model(ctx, model.clone(), transform, pass.clone(), false);
            }
        });

        batch.add_camera_data(&self.camera(), &self.light, pass);
    }
}
