use std::collections::HashMap;
use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_render_graph::{GfxContext, PassId, RenderObject, RenderPass};
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformPropData},
    geometry::{BoundingBox, Shapes},
    manager::ResourceManager,
    resource::ResourceLifetime,
};

use crate::{manager::MeshResourceManager, model::Model};

pub struct RenderBatch {
    pub(crate) render_list: Vec<RenderObject>,
    pub(crate) scene_data: HashMap<PassId, Vec<u8>>,
    pub(crate) mesh_resource_manager: MeshResourceManager,
    vertex_padding: bool,
}

impl RenderBatch {
    pub fn new(ctx: &GfxContext) -> Self {
        Self {
            render_list: Vec::new(),
            scene_data: HashMap::new(),
            mesh_resource_manager: MeshResourceManager::new(),
            vertex_padding: ctx.vertex_padding,
        }
    }

    pub fn reset(&mut self) {
        self.render_list.clear();
        self.scene_data.clear();
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

            let mut bind_groups = Vec::new();
            if let Some(bind_group) = material_binding {
                bind_groups.push(bind_group);
            }

            let vertex_attributes = match pass.vertex_attributes() {
                Some(vertex_attributes) => vertex_attributes,
                None => {
                    resource_manager
                        .get(&model.materials[material_id].material)
                        .properties
                        .vertex_attributes
                }
            };

            let (pipeline, is_transparent) = if let Some(material) = &material {
                let blending_enabled = resource_manager
                    .get(&material.material)
                    .properties
                    .blending_enabled;

                let pipeline = resource_manager.get_data(&material.material, ()).pipeline;
                let pipeline_data = resource_manager.get_data(&pipeline, ());

                (Some(pipeline_data.pipeline.clone()), blending_enabled)
            } else {
                (None, false)
            };

            let mesh_data = resource_manager.get_data(mesh, vertex_attributes);

            self.render_list.push(RenderObject {
                model_id: model.id,
                transform,
                pass: pass.clone(),
                pipeline,
                is_transparent,
                bind_groups,
                vertex_buffer: mesh_data.vertex_buffer.clone(),
                vertices_offset: mesh_data.vertices_offset,
                vertices_len: mesh_data.vertices_len,
                vertices_count: mesh_data.vertices_count,
                index_buffer: mesh_data.index_buffer.clone(),
                indices_offset: mesh_data.indices_offset,
                indices_len: mesh_data.indices_len,
            });
        }

        // self.render_stats.add_object(&render_object);
    }

    pub fn add_bounds(
        &mut self,
        resource_manager: &mut ResourceManager,
        bounding_box: BoundingBox,
        transform: Transform,
        pass: RenderPass,
    ) {
        let mesh = Shapes::bounding_box(bounding_box, self.vertex_padding);

        let model = Model::builder("box")
            .mesh(mesh, None, resource_manager, ResourceLifetime::Transient)
            .build(resource_manager);

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

    pub fn scene_data(&self, pass_id: PassId) -> Option<&[u8]> {
        self.scene_data.get(&pass_id).map(Vec::as_slice)
    }

    fn sort(&mut self) {
        self.render_list.sort_by(|a, b| {
            // sort order: pass, transparent, material, model
            (a.pass.id().cmp(&b.pass.id()))
                .then(a.is_transparent().cmp(&b.is_transparent()))
                .then(a.pipeline_id().cmp(&b.pipeline_id()))
                .then(a.model_id.cmp(&b.model_id))
        });
    }

    pub fn finish(&mut self) {
        self.sort();
    }
}
