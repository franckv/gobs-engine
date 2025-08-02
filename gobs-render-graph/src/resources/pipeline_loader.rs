use std::sync::Arc;

use gobs_gfx::{
    BindingGroupLayout, BindingGroupType, ComputePipelineBuilder, DescriptorStage,
    DynamicStateElem, GfxBindingGroupLayout, GfxDevice, GfxPipeline, GraphicsPipelineBuilder,
    Pipeline as _, Rect2D, Viewport,
};
use gobs_resource::{
    manager::ResourceRegistry,
    resource::{Resource, ResourceError, ResourceHandle, ResourceLoader},
};

use crate::resources::pipeline::{
    ComputePipelineProperties, GraphicsPipelineProperties, Pipeline, PipelineData,
    PipelineProperties,
};

pub struct PipelineLoader {
    device: Arc<GfxDevice>,
}

impl PipelineLoader {
    pub fn new(device: Arc<GfxDevice>) -> Self {
        Self { device }
    }

    pub fn load_compute(&self, properties: &ComputePipelineProperties) -> PipelineData {
        let mut pipeline = GfxPipeline::compute(&properties.name, self.device.clone());

        if let Some(shader) = &properties.compute_shader {
            pipeline = pipeline.shader(shader, &properties.compute_entry).unwrap();
        }

        for (_, bindings) in &properties.binding_groups {
            let mut binding_group_layout =
                GfxBindingGroupLayout::new(BindingGroupType::ComputeData);

            for binding in bindings {
                binding_group_layout =
                    binding_group_layout.add_binding(*binding, DescriptorStage::Compute);
            }

            pipeline = pipeline.binding_group(binding_group_layout);
        }

        PipelineData {
            pipeline: pipeline.build(),
        }
    }

    pub fn load_graphics(&self, properties: &GraphicsPipelineProperties) -> PipelineData {
        let mut pipeline = GfxPipeline::graphics(&properties.name, self.device.clone())
            .pool_size(properties.ds_pool_size)
            .push_constants(properties.push_constants)
            .vertex_attributes(properties.vertex_attributes)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .polygon_mode(properties.polygon_mode)
            .front_face(properties.front_face)
            .cull_mode(properties.cull_mode)
            .blending_enabled(properties.blend_mode)
            .attachments(properties.color_format, properties.depth_format);

        if let Some(shader) = &properties.vertex_shader {
            if let Some(entry) = &properties.vertex_entry {
                pipeline = pipeline.vertex_shader(shader, entry).unwrap();
            }
        }

        if let Some(shader) = &properties.fragment_shader {
            if let Some(entry) = &properties.fragment_entry {
                pipeline = pipeline.fragment_shader(shader, entry).unwrap();
            }
        }

        if properties.depth_test_enable {
            pipeline = pipeline
                .depth_test_enable(properties.depth_test_write_enable, properties.depth_test_op);
        } else {
            pipeline = pipeline.depth_test_disable();
        }

        for (stage, ty, bindings) in &properties.binding_groups {
            let mut binding_group_layout = GfxBindingGroupLayout::new(*ty);
            for binding in bindings {
                binding_group_layout = binding_group_layout.add_binding(*binding, *stage);
            }

            pipeline = pipeline.binding_group(binding_group_layout);
        }

        PipelineData {
            pipeline: pipeline.build(),
        }
    }
}

impl ResourceLoader<Pipeline> for PipelineLoader {
    fn load(
        &mut self,
        handle: &ResourceHandle<Pipeline>,
        _: &(),
        registry: &mut ResourceRegistry,
    ) -> Result<PipelineData, ResourceError> {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        let data = match &properties {
            PipelineProperties::Compute(properties) => self.load_compute(properties),
            PipelineProperties::Graphics(properties) => self.load_graphics(properties),
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<Pipeline>) {}
}
