use std::{collections::HashMap, sync::Arc};

use glam::Mat3;
use gobs_utils::timer::Timer;
use parking_lot::RwLock;
use uuid::Uuid;

use gobs_core::entity::{
    camera::Camera,
    light::Light,
    uniform::{UniformData, UniformLayout, UniformProp, UniformPropData},
};
use gobs_render::{
    context::Context,
    geometry::ModelId,
    graph::FrameGraph,
    pass::{PassId, RenderPass},
    renderable::{RenderObject, RenderStats, Renderable},
    resources::{ModelResource, UniformBuffer},
    CommandBuffer,
};
use gobs_vulkan::descriptor::{
    DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
};

use crate::graph::scenegraph::{NodeValue, SceneGraph};

struct SceneFrameData {
    pub uniform_ds: DescriptorSet,
    pub uniform_buffer: UniformBuffer,
}

impl SceneFrameData {
    pub fn new(
        ctx: &Context,
        uniform_layout: Arc<UniformLayout>,
        uniform_ds: DescriptorSet,
    ) -> Self {
        let uniform_buffer = UniformBuffer::new(
            ctx,
            uniform_ds.layout.clone(),
            uniform_layout.size(),
            ctx.allocator.clone(),
        );

        uniform_ds
            .update()
            .bind_buffer(&uniform_buffer.buffer, 0, uniform_buffer.buffer.size)
            .end();

        SceneFrameData {
            uniform_ds,
            uniform_buffer,
        }
    }
}

pub struct Scene {
    pub graph: SceneGraph,
    pub camera: Camera,
    pub light: Light,
    pub scene_data_layout: Arc<UniformLayout>,
    pub frame_number: usize,
    _scene_ds_pool: DescriptorSetPool,
    scene_frame_data: Vec<SceneFrameData>,
    model_manager: HashMap<(ModelId, PassId), Arc<ModelResource>>,
    stats: RwLock<RenderStats>,
    render_list: Vec<RenderObject>,
}

impl Scene {
    pub fn new(ctx: &Context, camera: Camera, light: Light) -> Self {
        let scene_descriptor_layout = DescriptorSetLayout::builder()
            .binding(DescriptorType::Uniform, DescriptorStage::All)
            .build(ctx.device.clone());

        let scene_data_layout = UniformLayout::builder()
            .prop("camera_position", UniformProp::Vec3F)
            .prop("view_proj", UniformProp::Mat4F)
            .prop("light_direction", UniformProp::Vec3F)
            .prop("light_color", UniformProp::Vec4F)
            .prop("ambient_color", UniformProp::Vec4F)
            .build();

        let mut _scene_ds_pool = DescriptorSetPool::new(
            ctx.device.clone(),
            scene_descriptor_layout.clone(),
            ctx.frames_in_flight as u32,
        );

        let scene_frame_data = (0..ctx.frames_in_flight)
            .map(|_| SceneFrameData::new(ctx, scene_data_layout.clone(), _scene_ds_pool.allocate()))
            .collect();

        Scene {
            graph: SceneGraph::new(),
            camera,
            light,
            scene_data_layout,
            frame_number: 0,
            _scene_ds_pool,
            scene_frame_data,
            model_manager: HashMap::new(),
            stats: RwLock::new(RenderStats::default()),
            render_list: Vec::new(),
        }
    }

    fn frame_id(&self, ctx: &Context) -> usize {
        (self.frame_number - 1) % ctx.frames_in_flight
    }

    pub fn update(&mut self, ctx: &Context, framegraph: &FrameGraph) {
        log::debug!("Update scene [{}]", self.frame_number);
        let timer = Timer::new();

        self.frame_number += 1;
        let frame_id = self.frame_id(ctx);

        let scene_data = UniformData::new(
            &self.scene_data_layout,
            &[
                UniformPropData::Vec3F(self.camera.position.into()),
                UniformPropData::Mat4F(self.camera.view_proj().to_cols_array_2d()),
                UniformPropData::Vec3F(self.light.position.normalize().into()),
                UniformPropData::Vec4F(self.light.colour.into()),
                UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]),
            ],
        );

        self.scene_frame_data[frame_id]
            .uniform_buffer
            .update(&scene_data);

        if self.frame_number % ctx.stats_refresh == 0 {
            self.stats.write().reset();
        }

        self.render_list.clear();

        self.graph.visit(self.graph.root, &mut |&transform, model| {
            if let NodeValue::Model(model) = model {
                if self.frame_number % ctx.stats_refresh == 0 {
                    self.stats.write().add_model(model);
                }

                for pass in [
                    framegraph.forward_pass.clone(),
                    framegraph.wire_pass.clone(),
                ] {
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
            }
        });

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

        if self.frame_number % ctx.stats_refresh == 0 {
            self.stats.write().update_time = timer.peek();
        }
    }
}

impl Renderable for Scene {
    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    fn draw(&self, ctx: &Context, pass: Arc<dyn RenderPass>, cmd: &CommandBuffer) {
        let timer = Timer::new();

        let frame_id = self.frame_id(ctx);

        let mut last_model = Uuid::nil();
        let mut last_material = Uuid::nil();
        let mut last_pipeline = Uuid::nil();

        let scene_data_ds = &self.scene_frame_data[frame_id].uniform_ds;

        let mut binds = 0;
        let mut draws = 0;

        for render_object in &self.render_list {
            if render_object.pass.id() != pass.id() {
                continue;
            }
            let world_matrix = render_object.transform.matrix;
            let normal_matrix = Mat3::from_quat(render_object.transform.rotation);
            let pipeline = match render_object.pass.pipeline() {
                Some(pipeline) => {
                    if last_pipeline != pipeline.id {
                        cmd.bind_pipeline(&pipeline);
                        last_pipeline = pipeline.id;
                        binds += 1;
                    }

                    pipeline
                }
                None => {
                    let material = &render_object.material;
                    let pipeline = material.pipeline();

                    if last_material != material.id {
                        if last_pipeline != pipeline.id {
                            cmd.bind_pipeline(&pipeline);
                            last_pipeline = pipeline.id;
                            binds += 1;
                        }
                        cmd.bind_descriptor_set(scene_data_ds, 0, &pipeline);
                        binds += 1;
                        if let Some(material_ds) = &material.material_ds {
                            cmd.bind_descriptor_set(material_ds, 1, &pipeline);
                            binds += 1;
                        }

                        last_material = material.id;
                    }

                    pipeline
                }
            };
            if let Some(push_layout) = render_object.pass.push_layout() {
                // TODO: hardcoded
                let model_data = UniformData::new(
                    &push_layout,
                    &[
                        UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                        UniformPropData::Mat3F(normal_matrix.to_cols_array_2d()),
                        UniformPropData::U64(
                            render_object
                                .model
                                .vertex_buffer
                                .address(ctx.device.clone()),
                        ),
                    ],
                );
                cmd.push_constants(pipeline.layout.clone(), &model_data.raw());
            }

            if last_model != render_object.model.model.id {
                cmd.bind_index_buffer::<u32>(
                    &render_object.model.index_buffer,
                    render_object.indices_offset,
                );
                last_model = render_object.model.model.id;
                binds += 1;
            }
            cmd.draw_indexed(render_object.indices_len, 1);
            draws += 1;
        }

        if self.frame_number % ctx.stats_refresh == 0 {
            self.stats.write().binds = binds;
            self.stats.write().draws = draws;
            self.stats.write().cpu_draw_time = timer.peek();
        }
    }

    fn stats(&self) -> RenderStats {
        self.stats.read().clone()
    }
}
