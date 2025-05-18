use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use gobs_gfx::{
    BindingGroup, BindingGroupType, BindingGroupUpdates, GfxBindingGroup, ImageLayout, Pipeline,
};
use gobs_resource::manager::ResourceManager;

use crate::{
    GfxContext,
    materials::{MaterialInstance, MaterialInstanceId},
};

pub struct MeshResourceManager {
    pub material_bindings: HashMap<MaterialInstanceId, GfxBindingGroup>,
}

impl MeshResourceManager {
    pub fn new() -> Self {
        Self {
            material_bindings: HashMap::new(),
        }
    }

    fn debug_stats(&self) {
        tracing::debug!(target: "render", "Bindings: {}", self.material_bindings.keys().len());
    }

    pub fn new_frame(&mut self, ctx: &GfxContext) {
        self.debug_stats();
    }

    /*
    #[tracing::instrument(target = "resources", skip_all, level = "trace")]
    fn load_object2(
        &mut self,
        ctx: &GfxContext,
        resource_manager: &mut ResourceManager,
        model: Arc<Model>,
        pass: RenderPass,
    ) -> Vec<MeshData> {
        let mut gpu_meshes = Vec::new();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        let mut vertices_offset;
        let mut indices_offset;

        let mut vertex_data = Vec::new();

        for (mesh, material_id) in &model.meshes {
            vertices_offset = vertices.len();
            indices_offset = indices.len();

            tracing::trace!(target: "render", "Vertex offset: {}, {}", vertices_offset, indices_offset);

            let vertex_attributes = match pass.vertex_attributes() {
                Some(vertex_attributes) => vertex_attributes,
                None => model.materials[material_id].vertex_attributes(),
            };
            // TODO: hot path
            let alignment = vertex_attributes.alignment();
            for vertice in &mesh.vertices {
                vertice.copy_data(vertex_attributes, alignment, &mut vertices);
            }
            for index in &mesh.indices {
                indices.push(*index);
            }

            vertex_data.push((
                material_id,
                vertices_offset as u64,
                vertices.len() - vertices_offset,
                mesh.vertices.len(),
                indices_offset,
                indices.len() - indices_offset,
            ))
        }

        let (vertex_buffer, index_buffer) = self.upload_vertices(ctx, &vertices, &indices);

        for (
            &material_id,
            vertices_offset,
            vertices_len,
            vertices_count,
            indices_offset,
            indices_len,
        ) in vertex_data
        {
            let material = model.materials.get(&material_id).cloned();

            // TODO: manage transient material
            let material_binding = self.load_material(resource_manager, material.clone());

            gpu_meshes.push(MeshData {
                ty: MeshPrimitiveType::Triangle,
                vertex_buffer: vertex_buffer.clone(),
                index_buffer: index_buffer.clone(),
                vertices_offset,
                vertices_len,
                vertices_count,
                indices_offset,
                indices_len,
            });
        }

        gpu_meshes
    }
    */

    pub(crate) fn load_material(
        &mut self,
        resource_manager: &mut ResourceManager,
        material: Option<Arc<MaterialInstance>>,
    ) -> Option<GfxBindingGroup> {
        if let Some(ref material) = material {
            tracing::debug!(target: "render", "Save binding for {}", material.id);

            match self.material_bindings.entry(material.id) {
                Entry::Vacant(e) => {
                    if !material.textures.is_empty() {
                        tracing::debug!(target: "render",
                            "Create material binding for pipeline: {}",
                            material.pipeline().id()
                        );
                        let binding = material
                            .material
                            .pipeline
                            .create_binding_group(BindingGroupType::MaterialData)
                            .unwrap();
                        let mut updater = binding.update();
                        for texture in &material.textures {
                            // TODO: load texture
                            let gpu_texture = resource_manager.get_data(texture, ());
                            updater = updater
                                .bind_sampled_image(&gpu_texture.image, ImageLayout::Shader)
                                .bind_sampler(&gpu_texture.sampler);
                        }
                        updater.end();

                        Some(e.insert(binding).clone())
                    } else {
                        None
                    }
                }
                Entry::Occupied(e) => Some(e.get().clone()),
            }
        } else {
            None
        }
    }
}
