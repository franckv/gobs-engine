use std::collections::HashMap;
use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformPropData},
    geometry::{BoundingBox, Shapes},
    manager::ResourceManager,
    resource::ResourceLifetime,
};

use crate::{
    GfxContext, RenderPass, manager::MeshResourceManager, model::Model, pass::PassId,
    renderable::RenderObject, stats::RenderStats,
};

pub struct RenderBatch {
    pub(crate) render_list: Vec<RenderObject>,
    pub(crate) scene_data: HashMap<PassId, Vec<u8>>,
    pub(crate) render_stats: RenderStats,
    pub(crate) mesh_resource_manager: MeshResourceManager,
}

impl RenderBatch {
    pub fn new() -> Self {
        Self {
            render_list: Vec::new(),
            scene_data: HashMap::new(),
            render_stats: RenderStats::default(),
            mesh_resource_manager: MeshResourceManager::new(),
        }
    }

    pub fn reset(&mut self) {
        self.render_list.clear();
        self.scene_data.clear();
        self.render_stats.reset();
        self.mesh_resource_manager.new_frame();
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    pub fn add_model(
        &mut self,
        resource_manager: &mut ResourceManager,
        model: Arc<Model>,
        transform: Transform,
        pass: RenderPass,
    ) {
        tracing::debug!(target: "render", "Add model: {}", model.meshes.len());

        for (mesh, material_id) in &model.meshes {
            let material = model.materials.get(material_id).cloned();
            let material_binding = self
                .mesh_resource_manager
                .load_material(resource_manager, material.clone());

            let vertex_attributes = match pass.vertex_attributes() {
                Some(vertex_attributes) => vertex_attributes,
                None => model.materials[material_id].vertex_attributes(),
            };

            let mesh_data = resource_manager.get_data(mesh, vertex_attributes);

            self.render_list.push(RenderObject {
                model_id: model.id,
                transform,
                pass: pass.clone(),
                mesh: mesh_data.clone(),
                material,
                material_binding,
            });
        }

        // self.render_stats.add_object(&render_object);
    }

    pub fn add_bounds(
        &mut self,
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
        bounding_box: BoundingBox,
        transform: Transform,
        pass: RenderPass,
    ) {
        let mesh = Shapes::bounding_box(bounding_box, ctx.vertex_padding);

        let model = Model::builder("box")
            .mesh(mesh, None, resource_manager, ResourceLifetime::Transient)
            .build();

        self.add_model(resource_manager, model, transform, pass);
    }

    pub fn add_camera_data(
        &mut self,
        camera: &Camera,
        camera_transform: &Transform,
        light: &Light,
        light_transform: &Transform,
        pass: RenderPass,
    ) {
        if pass.uniform_data_layout().is_some() {
            let scene_data =
                pass.get_uniform_data(camera, camera_transform, light, light_transform);
            self.scene_data.insert(pass.id(), scene_data);
        }
    }

    pub fn add_extent_data(&mut self, extent: ImageExtent2D, pass: RenderPass) {
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
            // sort order: pass, transparent, material, model
            (a.pass.id().cmp(&b.pass.id()))
                .then(a.is_transparent().cmp(&b.is_transparent()))
                .then(a.material_id().cmp(&b.material_id()))
                .then(a.model_id.cmp(&b.model_id))
        });
    }

    pub fn finish(&mut self) {
        self.sort();
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}
