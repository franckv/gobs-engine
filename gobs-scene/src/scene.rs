use std::{collections::HashMap, sync::Arc};

use gobs_core::entity::{
    camera::Camera,
    light::Light,
    uniform::{UniformData, UniformPropData},
};
use gobs_render::{
    context::Context,
    geometry::ModelId,
    pass::{PassId, RenderPass},
    renderable::{RenderBatch, RenderObject, Renderable},
    resources::ModelResource,
};
use gobs_utils::timer::Timer;

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

                    batch.add_object(render_object, false);
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
        let timer = Timer::new();

        self.generate_draw_list(ctx, pass.clone(), batch);

        if let Some(data_layout) = pass.uniform_data_layout() {
            batch.add_scene_data(
                UniformData::new(
                    &data_layout,
                    &[
                        UniformPropData::Vec3F(self.camera.position.into()),
                        UniformPropData::Mat4F(self.camera.view_proj().to_cols_array_2d()),
                        UniformPropData::Vec3F(self.light.position.normalize().into()),
                        UniformPropData::Vec4F(self.light.colour.into()),
                        UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]),
                    ],
                ),
                pass.id(),
            );
        }

        batch.stats_mut().update_time = timer.peek();
    }
}
