use std::sync::Arc;

use gobs_core::entity::{camera::Camera, light::Light};
use gobs_render::{
    context::Context,
    pass::RenderPass,
    renderable::{RenderBatch, Renderable},
};

use crate::graph::scenegraph::{NodeValue, SceneGraph};

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: Camera,
    pub light: Light,
}

impl Scene {
    pub fn new(camera: Camera, light: Light) -> Self {
        Scene {
            graph: SceneGraph::new(),
            camera,
            light,
        }
    }

    pub fn update(&mut self, _ctx: &Context, _delta: f32) {}
}

impl Renderable for Scene {
    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch) {
        self.graph.visit(self.graph.root, &mut |&transform, model| {
            if let NodeValue::Model(model) = model {
                batch.add_model(ctx, model.clone(), transform, pass.clone(), false);
            }
        });

        batch.add_camera_data(&self.camera, &self.light, pass);
    }
}
