use std::collections::HashMap;
use std::sync::Arc;

use gobs_core::{ImageExtent2D, Transform};
use gobs_render_graph::{GfxContext, PassId, RenderObject, RenderPass};
use gobs_resource::{
    entity::{camera::Camera, light::Light, uniform::UniformPropData},
    geometry::{BoundingBox, MeshBuilder, MeshGeometry, Shapes},
    manager::ResourceManager,
    resource::{ResourceError, ResourceLifetime},
};

use crate::{manager::MeshResourceManager, model::Model};

pub struct RenderBatch {
    pub(crate) render_list: Vec<RenderObject>,
    pub(crate) scene_data: HashMap<PassId, Vec<u8>>,
    pub(crate) mesh_resource_manager: MeshResourceManager,
    vertex_padding: bool,
    bounding_geometry: Option<MeshBuilder>,
    bounding_pass: Option<RenderPass>,
}

impl RenderBatch {
    pub fn new(ctx: &GfxContext) -> Self {
        Self {
            render_list: Vec::new(),
            scene_data: HashMap::new(),
            mesh_resource_manager: MeshResourceManager::new(),
            vertex_padding: ctx.vertex_padding,
            bounding_geometry: None,
            bounding_pass: None,
        }
    }

    pub fn reset(&mut self) {
        self.render_list.clear();
        self.scene_data.clear();
        self.mesh_resource_manager.new_frame();
        self.bounding_geometry = None;
    }

    #[tracing::instrument(target = "render", skip_all, level = "trace")]
    pub fn add_model(
        &mut self,
        resource_manager: &mut ResourceManager,
        model: Arc<Model>,
        transform: Transform,
        pass: RenderPass,
    ) -> Result<(), ResourceError> {
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

                let pipeline = resource_manager.get_data(&material.material, ())?.pipeline;
                let pipeline_data = resource_manager.get_data(&pipeline, ())?;

                (Some(pipeline_data.pipeline.clone()), blending_enabled)
            } else {
                (None, false)
            };

            let mesh_data = resource_manager.get_data(mesh, vertex_attributes)?;

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

        Ok(())
    }

    pub fn add_bounds(
        &mut self,
        bounding_box: BoundingBox,
        pass: RenderPass,
    ) -> Result<(), ResourceError> {
        let mesh = Shapes::bounding_box(bounding_box, self.vertex_padding);

        if self.bounding_geometry.is_none() {
            self.bounding_geometry = Some(MeshGeometry::builder("bounding"));
        }

        let builder = self.bounding_geometry.take();

        if let Some(builder) = builder {
            self.bounding_geometry = Some(builder.extend(mesh));
            self.bounding_pass = Some(pass);
        }

        Ok(())
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

    pub fn finish(&mut self, resource_manager: &mut ResourceManager) {
        let bb = self.bounding_geometry.take();

        if let Some(bb) = bb {
            let model = Model::builder("box")
                .mesh(
                    bb.build(),
                    None,
                    resource_manager,
                    ResourceLifetime::Transient,
                )
                .build(resource_manager);

            let pass = self.bounding_pass.take().unwrap();

            self.add_model(resource_manager, model, Transform::IDENTITY, pass)
                .expect("Add bounding box");
        }

        self.sort();
    }
}
