use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use gobs_core::logger;
use gobs_gfx::{
    BindingGroup, BindingGroupType, BindingGroupUpdates, GfxBindingGroup, ImageLayout, Pipeline,
};
use gobs_resource::manager::ResourceManager;

use crate::materials::{MaterialInstance, MaterialInstanceId};

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
        tracing::debug!(target: logger::RENDER, "Bindings: {}", self.material_bindings.keys().len());
    }

    pub fn new_frame(&mut self) {
        self.debug_stats();
    }

    pub(crate) fn load_material(
        &mut self,
        resource_manager: &mut ResourceManager,
        material: Option<Arc<MaterialInstance>>,
    ) -> Option<GfxBindingGroup> {
        if let Some(ref material) = material {
            tracing::debug!(target: logger::RENDER, "Save binding for {}", material.id);

            match self.material_bindings.entry(material.id) {
                Entry::Vacant(e) => {
                    if !material.textures.is_empty() {
                        let binding = Self::load_binding(resource_manager, material)?;

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

    fn load_binding(
        resource_manager: &mut ResourceManager,
        material: &MaterialInstance,
    ) -> Option<GfxBindingGroup> {
        let pipeline_handle = resource_manager
            .get_data(&material.material, ())
            .ok()?
            .pipeline;

        tracing::debug!(target: logger::RENDER,
            "Create material binding for pipeline: {:?}",
            pipeline_handle
        );

        let pipeline = &resource_manager
            .get_data(&pipeline_handle, ())
            .ok()?
            .pipeline;

        let binding = pipeline
            .create_binding_group(BindingGroupType::MaterialTextures)
            .unwrap();
        let mut updater = binding.update();
        for texture in &material.textures {
            // TODO: load texture
            let gpu_texture = resource_manager.get_data(texture, ()).ok()?;
            updater = updater
                .bind_sampled_image(&gpu_texture.image, ImageLayout::Shader)
                .bind_sampler(&gpu_texture.sampler);
        }
        updater.end();

        Some(binding)
    }
}
