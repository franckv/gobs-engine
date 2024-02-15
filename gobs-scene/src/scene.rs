use std::{collections::HashMap, sync::Arc};

use gobs_core::entity::{
    camera::Camera,
    light::Light,
    uniform::{UniformData, UniformPropData},
};
use gobs_render::{
    context::Context,
    geometry::ModelId,
    graph::FrameGraph,
    pass::{PassId, RenderPass},
    renderable::{RenderObject, RenderStats, Renderable},
    resources::ModelResource,
    CommandBuffer,
};
use gobs_utils::timer::Timer;

use crate::graph::scenegraph::{NodeValue, SceneGraph};

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: Camera,
    pub light: Light,
    pub frame_number: usize,
    model_manager: HashMap<(ModelId, PassId), Arc<ModelResource>>,
    render_list: Vec<RenderObject>,
}

impl Scene {
    pub fn new(camera: Camera, light: Light) -> Self {
        Scene {
            graph: SceneGraph::new(),
            camera,
            light,
            frame_number: 0,
            model_manager: HashMap::new(),
            render_list: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        framegraph: &FrameGraph,
        render_stats: &mut RenderStats,
    ) {
        let timer = Timer::new();

        self.render_list.clear();

        self.generate_draw_list(ctx, framegraph.forward_pass.clone());
        self.generate_draw_list(ctx, framegraph.wire_pass.clone());

        self.render_list.sort_by(|a, b| {
            // sort order: pass, material: model
            if a.pass.id() == b.pass.id() {
                if a.material.id == b.material.id {
                    a.model.model.id.cmp(&b.model.model.id)
                } else {
                    a.material.id.cmp(&b.material.id)
                }
            } else {
                a.pass.id().cmp(&b.pass.id())
            }
        });

        render_stats.update_time = timer.peek();
    }

    fn generate_draw_list(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>) {
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

                    self.render_list.push(render_object);
                }
            }
        });
    }
}

impl Renderable for Scene {
    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    fn draw(
        &self,
        ctx: &Context,
        pass: Arc<dyn RenderPass>,
        cmd: &CommandBuffer,
        render_stats: &mut RenderStats,
    ) {
        let scene_data = match pass.uniform_data_layout() {
            Some(data_layout) => Some(UniformData::new(
                &data_layout,
                &[
                    UniformPropData::Vec3F(self.camera.position.into()),
                    UniformPropData::Mat4F(self.camera.view_proj().to_cols_array_2d()),
                    UniformPropData::Vec3F(self.light.position.normalize().into()),
                    UniformPropData::Vec4F(self.light.colour.into()),
                    UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]),
                ],
            )),
            None => None,
        };

        pass.draw(ctx, cmd, &self.render_list, scene_data, render_stats);
    }
}
