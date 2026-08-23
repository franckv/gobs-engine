use std::sync::Arc;

use gobs_core::logger;
use gobs_render_graph::RenderFlags;
use gobs_render_hal::{
    BindResource, BindingGroupLayout, BindingGroupType, DescriptorType, Handle, RenderHAL,
};
use gobs_resource::{ResourceError, ResourceHandle, ResourceManager};

use crate::{
    GraphicsPipelineProperties, MaterialInstance, PipelineProperties, Texture,
    data::TextureDataProp,
};

pub struct MaterialSystem;

#[derive(Clone, Debug, Default)]
pub struct MaterialRenderData {
    pub render_flags: RenderFlags,
    pub pipeline: Option<Handle>,
    pub pipeline_properties: Option<GraphicsPipelineProperties>,
    pub material_data: Option<BindResource>,
    pub material_textures: Option<BindResource>,
    pub texture_indexing: bool,
}

impl MaterialSystem {
    pub fn get_pipeline(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        material_instance_handle: ResourceHandle<MaterialInstance>,
        render_flags: &mut RenderFlags,
    ) -> Result<(Handle, GraphicsPipelineProperties), ResourceError> {
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
            Ok((pipeline_data.data.pipeline, properties.clone()))
        } else {
            Err(ResourceError::InvalidData)
        }
    }

    pub fn get_material_data(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        material_instance_handle: ResourceHandle<MaterialInstance>,
    ) -> Result<MaterialRenderData, ResourceError> {
        let mut render_flags = RenderFlags::default();

        let (pipeline, pipeline_properties) = Self::get_pipeline(
            hal,
            resource_manager,
            material_instance_handle,
            &mut render_flags,
        )?;

        let (material_buffer, material_constant_data, material, textures) = {
            let resource_data = resource_manager.get_data(hal, &material_instance_handle)?;

            (
                resource_data.data.material_buffer,
                resource_data.properties.material_data.clone(),
                resource_data.properties.material,
                resource_data.properties.textures.clone(),
            )
        };

        let (texture_indexing, texture_data_layout) = {
            let material_properties = &resource_manager.get(&material).properties;

            (
                material_properties.texture_data_layout.texture_indexing,
                material_properties.texture_data_layout.layout.clone(),
            )
        };

        if texture_indexing && let Some(material_constant_data) = material_constant_data {
            for (prop, texture) in texture_data_layout.iter().zip(&textures) {
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

        let material_properties = &resource_manager.get(&material).properties;

        let material_data = material_properties
            .pipeline_properties
            .binding_groups
            .iter()
            .find(|group| group.binding_group_type == BindingGroupType::MaterialData)
            .cloned()
            .and_then(|layout| Self::get_material_data_binding(material_buffer, layout));

        let material_textures = if textures.is_empty() || texture_indexing {
            None
        } else {
            material_properties
                .pipeline_properties
                .binding_groups
                .iter()
                .find(|group| group.binding_group_type == BindingGroupType::MaterialTextures)
                .cloned()
                .and_then(|layout| {
                    Self::get_material_textures_binding(hal, resource_manager, textures, layout)
                })
        };

        Ok(MaterialRenderData {
            render_flags,
            pipeline: Some(pipeline),
            pipeline_properties: Some(pipeline_properties),
            material_data,
            material_textures,
            texture_indexing,
        })
    }

    pub fn get_material_data_binding(
        material_buffer: Option<Handle>,
        material_data_layout: Arc<BindingGroupLayout>,
    ) -> Option<BindResource> {
        material_buffer.map(|material_buffer| {
            BindResource::with_resources(material_data_layout, vec![material_buffer])
        })
    }

    pub fn get_material_textures_binding(
        hal: &mut dyn RenderHAL,
        resource_manager: &mut ResourceManager,
        textures: Vec<ResourceHandle<Texture>>,
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
}
