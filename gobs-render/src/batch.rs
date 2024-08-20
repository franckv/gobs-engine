use std::sync::Arc;
use std::{cmp::Ordering, collections::HashMap};

use gobs_core::{ImageExtent2D, Transform};
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformPropData},
    geometry::BoundingBox,
};

use crate::resources::MeshResourceManager;
use crate::Model;
use crate::{
    context::Context,
    pass::{PassId, RenderPass},
    renderable::RenderObject,
    stats::RenderStats,
};

pub struct RenderBatch {
    pub(crate) render_list: Vec<RenderObject>,
    pub(crate) scene_data: HashMap<PassId, Vec<u8>>,
    pub(crate) render_stats: RenderStats,
    pub(crate) mesh_resource_manager: MeshResourceManager,
}

impl RenderBatch {
    pub fn new(ctx: &Context) -> Self {
        Self {
            render_list: Vec::new(),
            scene_data: HashMap::new(),
            render_stats: RenderStats::default(),
            mesh_resource_manager: MeshResourceManager::new(ctx),
        }
    }

    pub fn reset(&mut self, ctx: &Context) {
        self.render_list.clear();
        self.scene_data.clear();
        self.render_stats.reset();
        self.mesh_resource_manager.new_frame(ctx);
    }

    pub fn add_model(
        &mut self,
        ctx: &Context,
        model: Arc<Model>,
        transform: Transform,
        pass: Arc<dyn RenderPass>,
        transient: bool,
    ) {
        log::debug!("Add model: {}", model.meshes.len());

        let mesh_data = self
            .mesh_resource_manager
            .add_object(ctx, model, pass.clone(), transient);

        for mesh in mesh_data {
            log::debug!("Add {} indices", mesh.indices_len);

            let render_object = RenderObject {
                transform,
                pass: pass.clone(),
                mesh: mesh.clone(),
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
        let mesh_data =
            self.mesh_resource_manager
                .add_bounding_box(ctx, bounding_box, pass.clone());

        for mesh in mesh_data {
            let render_object = RenderObject {
                transform,
                pass: pass.clone(),
                mesh: mesh.clone(),
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
        if let Some(_) = pass.uniform_data_layout() {
            let scene_data =
                pass.get_uniform_data(camera, camera_transform, light, light_transform);
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

    fn sort(&mut self) {
        self.render_list.sort_by(|a, b| {
            // sort order: pass, transparent, material: model
            if a.pass.id() != b.pass.id() {
                a.pass.id().cmp(&b.pass.id())
            } else if a.mesh.material.is_none() || b.mesh.material.is_none() {
                if a.mesh.material.is_some()
                    && a.mesh.material.clone().unwrap().material.blending_enabled
                {
                    Ordering::Greater
                } else if b.mesh.material.is_some()
                    && b.mesh.material.clone().unwrap().material.blending_enabled
                {
                    Ordering::Less
                } else {
                    a.mesh.model.id.cmp(&b.mesh.model.id)
                }
            } else if a.mesh.material.clone().unwrap().material.blending_enabled
                == b.mesh.material.clone().unwrap().material.blending_enabled
            {
                if a.mesh.material.clone().unwrap().id == b.mesh.material.clone().unwrap().id {
                    a.mesh.model.id.cmp(&b.mesh.model.id)
                } else {
                    a.mesh
                        .material
                        .clone()
                        .unwrap()
                        .id
                        .cmp(&b.mesh.material.clone().unwrap().id)
                }
            } else if a.mesh.material.clone().unwrap().material.blending_enabled {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
    }

    pub fn finish(&mut self) {
        self.sort();
    }
}
