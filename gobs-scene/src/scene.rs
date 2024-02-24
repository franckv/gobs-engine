use std::{collections::HashMap, sync::Arc};

use gobs_core::entity::{camera::Camera, light::Light};
use gobs_render::{
    context::Context,
    geometry::ModelId,
    pass::{PassId, RenderPass},
    renderable::{RenderBatch, RenderObject, Renderable},
    resources::ModelResource,
};

use crate::graph::scenegraph::{NodeValue, SceneGraph};

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: Camera,
    pub light: Light,
    model_manager: HashMap<(ModelId, PassId), Arc<ModelResource>>,
}

impl Scene {
    pub fn new(camera: Camera, light: Light) -> Self {
        Scene {
            graph: SceneGraph::new(),
            camera,
            light,
            model_manager: HashMap::new(),
        }
    }

    pub fn update(&mut self, _ctx: &Context, _delta: f32) {}

    fn generate_draw_list(
        &mut self,
        ctx: &Context,
        pass: Arc<dyn RenderPass>,
        batch: &mut RenderBatch,
    ) {
        self.graph.visit(self.graph.root, &mut |&transform, model| {
            if let NodeValue::Model(model) = model {
                let resource = self
                    .model_manager
                    .entry((model.id, pass.id()))
                    .or_insert_with(|| ModelResource::new(ctx, model.clone(), pass.clone()));

                for primitive in &resource.primitives {
                    let render_object = RenderObject {
                        transform,
                        pass: pass.clone(),
                        model: resource.clone(),
                        material: model.materials[&primitive.material].clone(),
                        indices_offset: primitive.offset,
                        indices_len: primitive.len,
                    };

                    batch.add_object(render_object);
                }
            }
        });
    }
}

impl Renderable for Scene {
    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch) {
        self.generate_draw_list(ctx, pass.clone(), batch);

        batch.add_camera_data(&self.camera, &self.light, pass);
    }
}
