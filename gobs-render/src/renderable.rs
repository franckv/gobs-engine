use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::{cmp::Ordering, collections::HashMap};

use gobs_core::{
    entity::{camera::Camera, light::Light, uniform::UniformPropData},
    Transform,
};
use gobs_vulkan::image::ImageExtent2D;

use crate::geometry::{BoundingBox, Model, ModelId};
use crate::{
    context::Context,
    material::MaterialInstance,
    pass::{PassId, PassType, RenderPass},
    resources::ModelResource,
    stats::RenderStats,
};

struct ModelManager {
    frame_number: usize,
    models: HashMap<(ModelId, PassId), Arc<ModelResource>>,
    transient_models: Vec<Vec<Arc<ModelResource>>>,
}

impl ModelManager {
    pub fn new(ctx: &Context) -> Self {
        Self {
            frame_number: 0,
            models: HashMap::new(),
            transient_models: (0..(ctx.frames_in_flight + 1)).map(|_| vec![]).collect(),
        }
    }

    fn new_frame(&mut self, ctx: &Context) -> usize {
        self.frame_number += 1;
        self.transient_models[(self.frame_number - 1) % (ctx.frames_in_flight + 1)].clear();
        (self.frame_number - 1) % (ctx.frames_in_flight + 1)
    }

    pub fn frame_id(&self, ctx: &Context) -> usize {
        (self.frame_number - 1) % (ctx.frames_in_flight + 1)
    }

    pub fn add_model(
        &mut self,
        ctx: &Context,
        model: Arc<Model>,
        pass: Arc<dyn RenderPass>,
        overwrite: bool,
    ) -> Arc<ModelResource> {
        let key = (model.id, pass.id());

        if overwrite {
            self.models.remove(&key);
        }
        match self.models.entry(key) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => entry
                .insert(ModelResource::new(ctx, model.clone(), pass.clone()))
                .clone(),
        }
    }

    pub fn add_bounding_box(
        &mut self,
        ctx: &Context,
        bounding_box: BoundingBox,
        pass: Arc<dyn RenderPass>,
    ) -> Arc<ModelResource> {
        let frame_id = self.frame_id(ctx);
        let model_manager = &mut self.transient_models[frame_id];

        model_manager.push(ModelResource::new_box(ctx, bounding_box, pass.clone()));

        model_manager.last().unwrap().clone()
    }
}

pub struct RenderBatch {
    pub(crate) render_list: Vec<RenderObject>,
    pub(crate) scene_data: HashMap<PassId, Vec<u8>>,
    pub(crate) render_stats: RenderStats,
    model_manager: ModelManager,
}

impl RenderBatch {
    pub fn new(ctx: &Context) -> Self {
        Self {
            render_list: Vec::new(),
            scene_data: HashMap::new(),
            render_stats: RenderStats::default(),
            model_manager: ModelManager::new(ctx),
        }
    }

    pub fn reset(&mut self, ctx: &Context) {
        self.render_list.clear();
        self.scene_data.clear();
        self.render_stats.reset();
        self.model_manager.new_frame(ctx);
    }

    pub fn add_model(
        &mut self,
        ctx: &Context,
        model: Arc<Model>,
        transform: Transform,
        pass: Arc<dyn RenderPass>,
        overwrite: bool,
    ) {
        let resource = self
            .model_manager
            .add_model(ctx, model, pass.clone(), overwrite);

        for primitive in &resource.primitives {
            let material = match primitive.material {
                Some(material) => Some(resource.model.materials[&material].clone()),
                None => None,
            };
            let render_object = RenderObject {
                transform,
                pass: pass.clone(),
                model: resource.clone(),
                material,
                vertices_offset: primitive.vertex_offset,
                indices_offset: primitive.index_offset,
                indices_len: primitive.len,
            };

            self.render_stats.add_object(&render_object);
            self.render_list.push(render_object);
        }
    }

    pub fn add_bounds(
        &mut self,
        ctx: &Context,
        bounding_box: BoundingBox,
        transform: Transform,
        pass: Arc<dyn RenderPass>,
    ) {
        let resource = self
            .model_manager
            .add_bounding_box(ctx, bounding_box, pass.clone());

        for primitive in &resource.primitives {
            let render_object = RenderObject {
                transform,
                pass: pass.clone(),
                model: resource.clone(),
                material: None,
                vertices_offset: primitive.vertex_offset,
                indices_offset: primitive.index_offset,
                indices_len: primitive.len,
            };

            self.render_stats.add_object(&render_object);
            self.render_list.push(render_object);
        }
    }

    pub fn add_camera_data(
        &mut self,
        camera: &Camera,
        camera_transform: &Transform,
        light: &Light,
        light_transform: &Transform,
        pass: Arc<dyn RenderPass>,
    ) {
        if let Some(data_layout) = pass.uniform_data_layout() {
            let scene_data = match pass.ty() {
                PassType::Bounds | PassType::Wire | PassType::Depth => {
                    data_layout.data(&[UniformPropData::Mat4F(
                        camera
                            .view_proj(camera_transform.translation)
                            .to_cols_array_2d(),
                    )])
                }
                _ => data_layout.data(&[
                    UniformPropData::Vec3F(camera_transform.translation.into()),
                    UniformPropData::Mat4F(
                        camera
                            .view_proj(camera_transform.translation)
                            .to_cols_array_2d(),
                    ),
                    UniformPropData::Vec3F(light_transform.translation.normalize().into()),
                    UniformPropData::Vec4F(light.colour.into()),
                    UniformPropData::Vec4F([0.1, 0.1, 0.1, 1.]),
                ]),
            };
            self.scene_data.insert(pass.id(), scene_data);
        }
    }

    pub fn add_extent_data(&mut self, extent: ImageExtent2D, pass: Arc<dyn RenderPass>) {
        if let Some(data_layout) = pass.uniform_data_layout() {
            let scene_data = data_layout.data(&[UniformPropData::Vec2F(extent.into())]);

            self.scene_data.insert(pass.id(), scene_data);
        }
    }

    pub fn scene_data(&self, pass_id: PassId) -> Option<&Vec<u8>> {
        self.scene_data.get(&pass_id)
    }

    pub fn stats_mut(&mut self) -> &mut RenderStats {
        &mut self.render_stats
    }

    pub fn finish(&mut self) {
        self.render_list.sort_by(|a, b| {
            // sort order: pass, transparent, material: model
            if a.pass.id() != b.pass.id() {
                a.pass.id().cmp(&b.pass.id())
            } else if a.material.is_none() || b.material.is_none() {
                if a.material.is_some() && a.material.clone().unwrap().material.blending_enabled {
                    Ordering::Greater
                } else if b.material.is_some()
                    && b.material.clone().unwrap().material.blending_enabled
                {
                    Ordering::Less
                } else {
                    a.model.model.id.cmp(&b.model.model.id)
                }
            } else if a.material.clone().unwrap().material.blending_enabled
                == b.material.clone().unwrap().material.blending_enabled
            {
                if a.material.clone().unwrap().id == b.material.clone().unwrap().id {
                    a.model.model.id.cmp(&b.model.model.id)
                } else {
                    a.material
                        .clone()
                        .unwrap()
                        .id
                        .cmp(&b.material.clone().unwrap().id)
                }
            } else if a.material.clone().unwrap().material.blending_enabled {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
    }
}

pub struct RenderObject {
    pub transform: Transform,
    pub pass: Arc<dyn RenderPass>,
    pub model: Arc<ModelResource>,
    pub material: Option<Arc<MaterialInstance>>,
    pub vertices_offset: u64,
    pub indices_offset: usize,
    pub indices_len: usize,
}

pub trait Renderable {
    fn resize(&mut self, width: u32, height: u32);
    fn draw(&mut self, ctx: &Context, pass: Arc<dyn RenderPass>, batch: &mut RenderBatch);
}
