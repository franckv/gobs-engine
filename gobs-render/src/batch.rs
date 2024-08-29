use std::collections::HashMap;
use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_gfx::Renderer;
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformPropData},
    geometry::BoundingBox,
};

use crate::model::Model;
use crate::{context::Context, resources::MeshResourceManager};
use crate::{
    pass::{PassId, RenderPass},
    renderable::RenderObject,
    stats::RenderStats,
};

pub struct RenderBatch<R: Renderer> {
    pub(crate) render_list: Vec<RenderObject<R>>,
    pub(crate) scene_data: HashMap<PassId, Vec<u8>>,
    pub(crate) render_stats: RenderStats,
    pub(crate) mesh_resource_manager: MeshResourceManager<R>,
}

impl<R: Renderer> RenderBatch<R> {
    pub fn new(ctx: &Context<R>) -> Self {
        Self {
            render_list: Vec::new(),
            scene_data: HashMap::new(),
            render_stats: RenderStats::default(),
            mesh_resource_manager: MeshResourceManager::new(ctx),
        }
    }

    pub fn reset(&mut self, ctx: &Context<R>) {
        self.render_list.clear();
        self.scene_data.clear();
        self.render_stats.reset();
        self.mesh_resource_manager.new_frame(ctx);
    }

    #[tracing::instrument(target = "render", skip_all, level = "debug")]
    pub fn add_model(
        &mut self,
        ctx: &Context<R>,
        model: Arc<Model<R>>,
        transform: Transform,
        pass: Arc<dyn RenderPass<R>>,
        transient: bool,
    ) {
        tracing::debug!("Add model: {}", model.meshes.len());

        let mesh_data = self
            .mesh_resource_manager
            .add_object(ctx, model, pass.clone(), transient);

        for mesh in mesh_data {
            tracing::debug!("Add {} indices", mesh.indices_len);

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
        ctx: &Context<R>,
        bounding_box: BoundingBox,
        transform: Transform,
        pass: Arc<dyn RenderPass<R>>,
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
        pass: Arc<dyn RenderPass<R>>,
    ) {
        if let Some(_) = pass.uniform_data_layout() {
            let scene_data =
                pass.get_uniform_data(camera, camera_transform, light, light_transform);
            self.scene_data.insert(pass.id(), scene_data);
        }
    }

    pub fn add_extent_data(&mut self, extent: ImageExtent2D, pass: Arc<dyn RenderPass<R>>) {
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
            return (a.pass.id().cmp(&b.pass.id()))
                .then(a.is_transparent().cmp(&b.is_transparent()))
                .then(a.material_id().cmp(&b.material_id()))
                .then(a.mesh.model.id.cmp(&b.mesh.model.id));
        });
    }

    pub fn finish(&mut self) {
        self.sort();
    }
}
