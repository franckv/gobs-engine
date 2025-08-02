use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use gobs_core::logger;
use gobs_gfx::{
    BindingGroupPool, GfxBindingGroup, GfxBindingGroupLayout, GfxBindingGroupPool, GfxDevice,
};
use gobs_resource::{
    manager::ResourceRegistry,
    resource::{
        Resource, ResourceError, ResourceHandle, ResourceId, ResourceLoader, ResourceProperties,
    },
};

use crate::{
    MaterialProperties,
    resources::{
        Material, MaterialInstance, MaterialInstanceData,
        material_instance::material_instance::MaterialInstanceProperties,
    },
};

pub struct MaterialInstanceLoader {
    device: Arc<GfxDevice>,
    pub material_bindings: HashMap<ResourceId, (Option<GfxBindingGroup>, Option<GfxBindingGroup>)>,
    material_binding_pools: HashMap<ResourceId, GfxBindingGroupPool>,
    texture_binding_pools: HashMap<ResourceId, GfxBindingGroupPool>,
}

impl MaterialInstanceLoader {
    pub fn new(device: Arc<GfxDevice>) -> Self {
        Self {
            device: device.clone(),
            material_bindings: HashMap::new(),
            material_binding_pools: HashMap::new(),
            texture_binding_pools: HashMap::new(),
        }
    }
}

impl ResourceLoader<MaterialInstance> for MaterialInstanceLoader {
    fn load(
        &mut self,
        handle: &ResourceHandle<MaterialInstance>,
        _parameter: &(),
        registry: &mut ResourceRegistry,
    ) -> Result<MaterialInstanceData, ResourceError> {
        let resource = registry.get(handle);
        let properties = &resource.properties;

        let material_handle = properties.material;
        let material = registry.get(&material_handle);

        Self::init_material_pools(
            self.device.clone(),
            &mut self.material_binding_pools,
            material.properties.material_data_layout.bindings_layout(),
            handle,
            properties,
            material,
        );

        Self::init_material_pools(
            self.device.clone(),
            &mut self.texture_binding_pools,
            material.properties.texture_data_layout.bindings_layout(),
            handle,
            properties,
            material,
        );

        let (material_binding, texture_binding) =
            self.load_material_bindings(handle, &material.properties);

        let data = MaterialInstanceData {
            material: properties.material,
            texture_binding,
            material_binding,
            bound: false,
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<MaterialInstance>) {
        todo!()
    }
}

impl MaterialInstanceLoader {
    fn init_material_pools(
        device: Arc<GfxDevice>,
        pools: &mut HashMap<ResourceId, GfxBindingGroupPool>,
        binding_layout: GfxBindingGroupLayout,
        handle: &ResourceHandle<MaterialInstance>,
        properties: &MaterialInstanceProperties,
        material: &Resource<Material>,
    ) {
        match pools.entry(handle.id) {
            Entry::Occupied(_pools) => {
                tracing::warn!(target: logger::RESOURCES, "Pool already initialized for material instance {}", properties.name());
            }
            Entry::Vacant(pools) => {
                let pool_size = material.properties.pipeline_properties.ds_pool_size;
                let pool = GfxBindingGroupPool::new(device, pool_size, binding_layout);

                pools.insert(pool);
            }
        }
    }

    fn load_material_bindings(
        &mut self,
        handle: &ResourceHandle<MaterialInstance>,
        material_properties: &MaterialProperties,
    ) -> (Option<GfxBindingGroup>, Option<GfxBindingGroup>) {
        match self.material_bindings.entry(handle.id) {
            Entry::Occupied(e) => {
                let (material_binding, texture_binding) = e.get().clone();
                (material_binding, texture_binding)
            }
            Entry::Vacant(e) => {
                let texture_binding = Self::load_textures(
                    material_properties,
                    self.texture_binding_pools.get_mut(&handle.id).unwrap(),
                );

                let material_binding = Self::load_material(material_properties);

                e.insert((material_binding, texture_binding)).clone()
            }
        }
    }

    fn load_textures(
        material_properties: &MaterialProperties,
        pool: &mut GfxBindingGroupPool,
    ) -> Option<GfxBindingGroup> {
        if !material_properties.texture_data_layout.is_empty() {
            let binding = pool.allocate();

            // let mut update = binding.update();
            // for texture in &properties.textures {
            //     update = update
            //         .bind_sampled_image(image, gobs_gfx::ImageLayout::Shader)
            //         .bind_sampler(sampler);
            // }

            Some(binding)
        } else {
            None
        }

        // for texture in &material.textures {
        //     // TODO: load texture
        //     let gpu_texture = resource_manager.get_data(texture, ()).ok()?;
        //     updater = updater
        //         .bind_sampled_image(&gpu_texture.image, ImageLayout::Shader)
        //         .bind_sampler(&gpu_texture.sampler);
        // }
        // updater.end();
    }

    fn load_material(material_properties: &MaterialProperties) -> Option<GfxBindingGroup> {
        if !material_properties.material_data_layout.is_empty() {
            None
        } else {
            None
        }
    }
}
