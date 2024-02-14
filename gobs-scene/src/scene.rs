use std::sync::{Arc, RwLock};

use glam::Mat3;
use gobs_utils::timer::Timer;
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
    CommandBuffer,
};
use gobs_vulkan::descriptor::{
    DescriptorSet, DescriptorSetLayout, DescriptorSetPool, DescriptorStage, DescriptorType,
};

use crate::{
    graph::scenegraph::{NodeValue, SceneGraph},
    manager::ResourceManager,
    renderable::{RenderStats, Renderable},
    resources::{ModelResource, UniformBuffer},
};

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
    model_manager: ResourceManager<(ModelId, PassId), Arc<ModelResource>>,
    stats: RwLock<RenderStats>,
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
            model_manager: ResourceManager::new(),
            stats: RwLock::new(RenderStats::default()),
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

        self.stats.write().unwrap().reset();

        self.graph.visit(self.graph.root, &mut |_, model| {
            if let NodeValue::Model(model) = model {
                self.stats.write().unwrap().add_model(model);

                self.model_manager.add(
                    ctx,
                    model.id,
                    framegraph.forward_pass.clone(),
                    model.clone(),
                );
                self.model_manager
                    .add(ctx, model.id, framegraph.wire_pass.clone(), model.clone());
            }
        });

        self.stats.write().unwrap().update_time = timer.peek();
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
        self.graph
            .visit_sorted(self.graph.root, &mut |transform, model| {
                if let NodeValue::Model(model) = model {
                    let world_matrix = transform.matrix;
                    let normal_matrix = Mat3::from_quat(transform.rotation);

                    let model = self.model_manager.get(model.id, pass.id());

                    for primitive in &model.primitives {
                        let pipeline = match pass.pipeline() {
                            Some(pipeline) => {
                                if last_pipeline != pipeline.id {
                                    cmd.bind_pipeline(&pipeline);
                                    last_pipeline = pipeline.id;
                                }

                                pipeline
                            }
                            None => {
                                let material = &model.model.materials[&primitive.material];
                                let pipeline = material.pipeline();

                                if last_material != material.id {
                                    if last_pipeline != pipeline.id {
                                        cmd.bind_pipeline(&pipeline);
                                        last_pipeline = pipeline.id;
                                    }
                                    cmd.bind_descriptor_set(scene_data_ds, 0, &pipeline);
                                    if let Some(material_ds) = &material.material_ds {
                                        cmd.bind_descriptor_set(material_ds, 1, &pipeline);
                                    }

                                    last_material = material.id;
                                }

                                pipeline
                            }
                        };

                        if let Some(push_layout) = pass.push_layout() {
                            // TODO: hardcoded
                            let model_data = UniformData::new(
                                &push_layout,
                                &[
                                    UniformPropData::Mat4F(world_matrix.to_cols_array_2d()),
                                    UniformPropData::Mat3F(normal_matrix.to_cols_array_2d()),
                                    UniformPropData::U64(
                                        model.vertex_buffer.address(ctx.device.clone()),
                                    ),
                                ],
                            );
                            cmd.push_constants(pipeline.layout.clone(), &model_data.raw());
                        }

                        if last_model != model.model.id {
                            cmd.bind_index_buffer::<u32>(&model.index_buffer, primitive.offset);
                            last_model = model.model.id;
                        }
                        cmd.draw_indexed(primitive.len, 1);
                    }
                }
            });

        self.stats.write().unwrap().cpu_draw_time = timer.peek();
    }

    fn stats(&self) -> RenderStats {
        self.stats.read().unwrap().clone()
    }
}
