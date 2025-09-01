use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use gobs_core::logger;
use gobs_gfx::{
    BindingGroup, BindingGroupPool, BindingGroupUpdates, Buffer, BufferUsage, GfxBindingGroup,
    GfxBindingGroupLayout, GfxBindingGroupPool, GfxBuffer, GfxDevice,
};
use gobs_render_low::{MaterialConstantData, MaterialDataLayout, MaterialInstanceId, UniformData};
use gobs_resource::{
    manager::ResourceRegistry,
    resource::{Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
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
    pub material_bindings:
        HashMap<MaterialInstanceId, (Option<GfxBindingGroup>, Option<GfxBindingGroup>)>,
    material_binding_pools: HashMap<MaterialInstanceId, GfxBindingGroupPool>,
    texture_binding_pools: HashMap<MaterialInstanceId, GfxBindingGroupPool>,
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
            properties,
            material,
        );

        Self::init_material_pools(
            self.device.clone(),
            &mut self.texture_binding_pools,
            material.properties.texture_data_layout.bindings_layout(),
            properties,
            material,
        );

        let material_buffer = self.create_buffer(
            properties.name(),
            &properties.material_data_layout,
            properties.material_data.as_ref(),
        );

        let (material_binding, texture_binding) =
            self.load_material_bindings(properties, material_buffer.as_ref(), &material.properties);

        let data = MaterialInstanceData {
            material: properties.material,
            material_buffer,
            texture_binding,
            material_binding,
            bound: false,
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<MaterialInstance>) {}
}

impl MaterialInstanceLoader {
    fn init_material_pools(
        device: Arc<GfxDevice>,
        pools: &mut HashMap<MaterialInstanceId, GfxBindingGroupPool>,
        binding_layout: GfxBindingGroupLayout,
        properties: &MaterialInstanceProperties,
        material: &Resource<Material>,
    ) {
        match pools.entry(properties.id) {
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

    fn create_buffer(
        &self,
        name: &str,
        material_data_layout: &MaterialDataLayout,
        material_data: Option<&MaterialConstantData>,
    ) -> Option<GfxBuffer> {
        let mut data = Vec::new();

        if let Some(material_data) = material_data {
            material_data_layout.copy_data(None, material_data, &mut data);

            let mut buffer = GfxBuffer::new(name, data.len(), BufferUsage::Uniform, &self.device);
            buffer.copy(&data, 0);

            Some(buffer)
        } else {
            None
        }
    }

    fn load_material_bindings(
        &mut self,
        properties: &MaterialInstanceProperties,
        material_buffer: Option<&GfxBuffer>,
        material_properties: &MaterialProperties,
    ) -> (Option<GfxBindingGroup>, Option<GfxBindingGroup>) {
        match self.material_bindings.entry(properties.id) {
            Entry::Occupied(e) => {
                let (material_binding, texture_binding) = e.get().clone();
                (material_binding, texture_binding)
            }
            Entry::Vacant(e) => {
                let texture_binding = Self::load_textures(
                    material_properties,
                    self.texture_binding_pools.get_mut(&properties.id).unwrap(),
                );

                let material_binding = Self::load_material(
                    material_buffer,
                    self.material_binding_pools.get_mut(&properties.id).unwrap(),
                );

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

            Some(binding)
        } else {
            None
        }
    }

    fn load_material(
        material_buffer: Option<&GfxBuffer>,
        pool: &mut GfxBindingGroupPool,
    ) -> Option<GfxBindingGroup> {
        if let Some(material_buffer) = material_buffer {
            tracing::debug!(target: logger::RESOURCES, "Bind material uniform buffer");
            let binding = pool.allocate();

            binding
                .update()
                .bind_buffer(material_buffer, 0, material_buffer.size())
                .end();

            Some(binding)
        } else {
            None
        }
    }
}
