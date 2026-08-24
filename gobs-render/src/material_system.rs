use std::sync::Arc;

use gobs_core::logger;
use gobs_render_graph::{MaterialRenderData, RenderFlags, SceneDataLayout};
use gobs_render_hal::{
    AttributeData, BindResource, BindingGroupLayout, BindingGroupType, BufferType, DescriptorType,
    Handle, RenderHAL, UniformData as _,
};
use gobs_resource::{ResourceError, ResourceHandle, ResourceManager, ResourceProperties as _};

use crate::{
    MaterialDataPropData, MaterialInstance, MaterialInstanceProperties, MaterialProperties,
    PipelineProperties, Texture,
    data::{
        MaterialConstantData, MaterialDataLayout, MaterialDataProp, TextureDataLayout,
        TextureDataProp,
    },
};

#[derive(Clone)]
pub enum MaterialDataBinding {
    None,
    Binded(BindResource),
    Bindless(usize),
}

#[derive(Clone)]
pub struct MaterialBinding {
    material_data_binding: MaterialDataBinding,
    texture_data_layout: TextureDataLayout,
    texture_binding_group_layout: Option<Arc<BindingGroupLayout>>,
    scene_layout: Option<SceneDataLayout>,
}

pub struct MaterialSystem;

impl MaterialSystem {
    pub fn get_material_data(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        material_instance_handle: ResourceHandle<MaterialInstance>,
    ) -> Result<MaterialRenderData, ResourceError> {
        let mut material_render_flags = RenderFlags::default();

        let pipeline = Self::get_pipeline(
            hal,
            resource_manager,
            material_instance_handle,
            &mut material_render_flags,
        )?;

        let resource_data = resource_manager.get_data(hal, &material_instance_handle)?;

        let material_binding = resource_data.data.material_binding.clone();
        let material_constant_data = resource_data.properties.material_data.clone();

        let textures = resource_data.properties.textures.clone();
        let texture_indexing = material_binding.texture_data_layout.texture_indexing;

        if texture_indexing && let Some(material_constant_data) = material_constant_data {
            for (prop, texture) in material_binding
                .texture_data_layout
                .layout
                .iter()
                .zip(&textures)
            {
                let handle = resource_manager.get_data(&mut *hal, texture)?.data.image;

                let index = match prop {
                    TextureDataProp::Diffuse => material_constant_data.diffuse_index,
                    TextureDataProp::Normal => material_constant_data.normal_index,
                    TextureDataProp::Emission => material_constant_data.emission_index,
                    TextureDataProp::Specular => material_constant_data.specular_index,
                };

                hal.update_texture_index(index as usize, handle);

                tracing::debug!(target: logger::RENDER, "Update texture {:?} with index {}", handle, index);
            }
        }

        let material_textures = if textures.is_empty() || texture_indexing {
            None
        } else {
            Self::get_material_textures_binding(
                hal,
                resource_manager,
                &textures,
                material_binding
                    .texture_binding_group_layout
                    .expect("Material with textures but no layout"),
            )
        };

        let (material_indexing, material_offset) =
            if let MaterialDataBinding::Bindless(index) = material_binding.material_data_binding {
                let offset = hal
                    .get_material_offset(index)
                    .expect("Material offset is None");
                (true, Some(offset))
            } else {
                (false, None)
            };

        let material_data = if let MaterialDataBinding::Binded(material_data) =
            material_binding.material_data_binding
        {
            Some(material_data)
        } else {
            None
        };

        Ok(MaterialRenderData {
            material_render_flags,
            pipeline: Some(pipeline),
            material_data,
            material_textures,
            material_offset,
            scene_layout: material_binding.scene_layout,
            texture_indexing,
            material_indexing,
        })
    }

    fn get_pipeline(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        material_instance_handle: ResourceHandle<MaterialInstance>,
        render_flags: &mut RenderFlags,
    ) -> Result<Handle, ResourceError> {
        let material_instance = resource_manager.get(&material_instance_handle);
        let material_handle = material_instance.properties.material;
        let material = resource_manager.get(&material_handle);

        if material.properties.blending_enabled {
            *render_flags |= RenderFlags::TRANSPARENT;
        } else {
            *render_flags |= RenderFlags::OPAQUE;
        }

        let material_data = resource_manager.get_data(hal, &material_handle)?;

        let pipeline_handle = material_data.data.pipeline;

        let pipeline_data = resource_manager.get_data(hal, &pipeline_handle)?;
        let pipeline_properties = pipeline_data.properties;

        if let PipelineProperties::Graphics(properties) = pipeline_properties {
            tracing::trace!(target: logger::RENDER, "Using pipeline {:?}", properties);
            Ok(pipeline_data.data.pipeline)
        } else {
            Err(ResourceError::InvalidData)
        }
    }

    fn get_material_data_binding(
        material_buffer: Option<Handle>,
        material_data_layout: Arc<BindingGroupLayout>,
    ) -> Option<BindResource> {
        material_buffer.map(|material_buffer| {
            BindResource::with_resources(material_data_layout, vec![material_buffer])
        })
    }

    fn get_material_textures_binding(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        textures: &[ResourceHandle<Texture>],
        texture_data_layout: Arc<BindingGroupLayout>,
    ) -> Option<BindResource> {
        let tex_data = textures
            .iter()
            .map(|t| {
                let data = resource_manager.get_data(&mut *hal, t)?;

                Ok((data.data.image, data.data.sampler))
            })
            .collect::<Result<Vec<_>, ResourceError>>()
            .ok()?;

        let mut texture_idx = 0;
        let mut sampler_idx = 0;

        let mut resource = BindResource::new(texture_data_layout.clone());

        for (ty, _, count) in &texture_data_layout.bindings {
            resource = resource.next();
            match ty {
                DescriptorType::SampledImage => {
                    let to_write = (tex_data.len() - texture_idx).min(*count as usize);

                    for i in 0..to_write {
                        resource = resource.binding(tex_data[texture_idx + i].0, i)
                    }
                    texture_idx += to_write;
                }
                DescriptorType::Sampler => {
                    let to_write = (tex_data.len() - sampler_idx).min(*count as usize);

                    for i in 0..to_write {
                        resource = resource.binding(tex_data[sampler_idx + i].1, i)
                    }
                    sampler_idx += to_write;
                }
                _ => unimplemented!(),
            }
        }

        Some(resource)
    }

    pub fn get_material_binding(
        hal: &mut dyn RenderHAL,
        material_instance_properties: &mut MaterialInstanceProperties,
        material_properties: &MaterialProperties,
    ) -> MaterialBinding {
        #[cfg(debug_assertions)]
        Self::validate_material_layout(material_instance_properties, material_properties);

        let texture_layout = &material_properties.texture_data_layout;
        let material_layout = &material_properties.material_data_layout;

        if texture_layout.texture_indexing {
            for &texture_prop in &texture_layout.layout {
                let index = hal.allocate_texture_index() as u32;

                tracing::debug!(target: logger::RESOURCES, "Alloc texture index {} for {:?}", index, texture_prop);

                let prop = match texture_prop {
                    TextureDataProp::Diffuse => MaterialDataPropData::DiffuseIndex(index),
                    TextureDataProp::Normal => MaterialDataPropData::NormalIndex(index),
                    TextureDataProp::Emission => MaterialDataPropData::EmissionIndex(index),
                    TextureDataProp::Specular => MaterialDataPropData::SpecularIndex(index),
                };

                material_instance_properties.add_prop(prop);
            }
        }

        let material_data_binding = if material_layout.material_indexing {
            let index = Self::upload_material_data(
                hal,
                &material_properties.material_data_layout,
                material_instance_properties.material_data.as_ref(),
            );

            if let Some(index) = index {
                MaterialDataBinding::Bindless(index)
            } else {
                MaterialDataBinding::None
            }
        } else {
            let material_buffer = Self::create_material_buffer(
                hal,
                material_instance_properties.name(),
                &material_properties.material_data_layout,
                material_instance_properties.material_data.as_ref(),
            );

            let material_data_binding = material_properties
                .pipeline_properties
                .binding_groups
                .iter()
                .find(|group| group.binding_group_type == BindingGroupType::MaterialData)
                .cloned()
                .and_then(|layout| Self::get_material_data_binding(material_buffer, layout));

            if let Some(material_data_binding) = material_data_binding {
                MaterialDataBinding::Binded(material_data_binding)
            } else {
                MaterialDataBinding::None
            }
        };

        let texture_binding_group_layout = material_properties
            .pipeline_properties
            .binding_groups
            .iter()
            .find(|group| group.binding_group_type == BindingGroupType::MaterialTextures)
            .cloned();

        let scene_layout = material_properties
            .pipeline_properties
            .scene_data_layout
            .clone();

        MaterialBinding {
            material_data_binding,
            texture_data_layout: material_properties.texture_data_layout.clone(),
            texture_binding_group_layout,
            scene_layout: Some(scene_layout),
        }
    }

    #[cfg(debug_assertions)]
    fn validate_material_layout(
        properties: &MaterialInstanceProperties,
        material_properties: &MaterialProperties,
    ) {
        if properties.material_data.is_none()
            && !material_properties.material_data_layout.is_empty()
        {
            tracing::error!(target: logger::RESOURCES, "Material instance does not contain material data");
            panic!("Failed to load material instance: {}", properties.name);
        }
    }

    fn collect_material_data(
        material_data_layout: &MaterialDataLayout,
        material_data: &MaterialConstantData,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        material_data_layout.copy_data(&mut data, |prop| match prop {
            MaterialDataProp::DiffuseColor => AttributeData::Vec4F(material_data.diffuse_color),
            MaterialDataProp::EmissionColor => AttributeData::Vec4F(material_data.emission_color),
            MaterialDataProp::SpecularColor => AttributeData::Vec4F(material_data.specular_color),
            MaterialDataProp::SpecularPower => AttributeData::F32(material_data.specular_power),
            MaterialDataProp::DiffuseIndex => AttributeData::U32(material_data.diffuse_index),
            MaterialDataProp::NormalIndex => AttributeData::U32(material_data.normal_index),
            MaterialDataProp::EmissionIndex => AttributeData::U32(material_data.emission_index),
            MaterialDataProp::SpecularIndex => AttributeData::U32(material_data.specular_index),
        });

        tracing::debug!(target: logger::RESOURCES, "Create material data buffer {:?}", &data);

        data
    }

    fn upload_material_data(
        hal: &mut dyn RenderHAL,
        material_data_layout: &MaterialDataLayout,
        material_data: Option<&MaterialConstantData>,
    ) -> Option<usize> {
        if let Some(material_data) = material_data {
            let data = Self::collect_material_data(material_data_layout, material_data);

            let index = hal.allocate_material_index();

            hal.update_material_data(index, &data);

            Some(index)
        } else {
            None
        }
    }

    fn create_material_buffer(
        hal: &mut dyn RenderHAL,
        name: &str,
        material_data_layout: &MaterialDataLayout,
        material_data: Option<&MaterialConstantData>,
    ) -> Option<Handle> {
        if let Some(material_data) = material_data {
            let data = Self::collect_material_data(material_data_layout, material_data);

            let buffer = hal.create_buffer(name, data.len(), BufferType::Uniform);
            hal.upload_buffer(buffer, &data, 0);

            Some(buffer)
        } else {
            None
        }
    }

    pub(crate) fn destroy_material_binding(
        hal: &mut dyn RenderHAL,
        material_binding: MaterialBinding,
        material_data: Option<MaterialConstantData>,
    ) {
        match &material_binding.material_data_binding {
            MaterialDataBinding::Binded(resource) => {
                let handle = resource.slot(0);
                if let Some(buffer) = handle {
                    hal.destroy_buffer(buffer);
                }
            }
            MaterialDataBinding::Bindless(index) => {
                hal.release_material_index(*index);
            }
            MaterialDataBinding::None => (),
        }

        if material_binding.texture_data_layout.texture_indexing
            && let Some(data) = material_data
        {
            for prop in material_binding.texture_data_layout.layout {
                let index = match prop {
                    TextureDataProp::Diffuse => data.diffuse_index,
                    TextureDataProp::Normal => data.normal_index,
                    TextureDataProp::Emission => data.emission_index,
                    TextureDataProp::Specular => data.specular_index,
                };

                hal.release_texture_index(index as usize);
            }
        }
    }
}
