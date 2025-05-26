use std::sync::Arc;

use gobs_gfx::{
    BindingGroupType, ComputePipelineBuilder, DynamicStateElem, GfxDevice, GfxPipeline,
    GraphicsPipelineBuilder, Pipeline as _, Rect2D, Viewport,
};
use gobs_resource::{
    manager::ResourceRegistry,
    resource::{Resource, ResourceHandle, ResourceLoader},
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
        let mut pipeline = GfxPipeline::compute(&properties.name, &self.device);

        if let Some(shader) = &properties.compute_shader {
            pipeline = pipeline.shader(shader, &properties.compute_entry).unwrap();
        }

        for (_, bindings) in &properties.binding_groups {
            pipeline = pipeline.binding_group(BindingGroupType::ComputeData);
            for binding in bindings {
                pipeline = pipeline.binding(*binding);
            }
        }

        PipelineData {
            pipeline: pipeline.build(),
        }
    }

    pub fn load_graphics(&self, properties: &GraphicsPipelineProperties) -> PipelineData {
        let mut pipeline = GfxPipeline::graphics(&properties.name, &self.device);

        pipeline = pipeline
            .pool_size(properties.ds_pool_size)
            .push_constants(properties.push_constants)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .front_face(properties.front_face)
            .cull_mode(properties.cull_mode)
            .blending_enabled(properties.blend_mode)
            .attachments(properties.color_format, properties.depth_format);

        if let Some(shader) = &properties.vertex_shader {
            pipeline = pipeline
                .vertex_shader(shader, &properties.vertex_entry)
                .unwrap();
        }

        if let Some(shader) = &properties.fragment_shader {
            pipeline = pipeline
                .fragment_shader(shader, &properties.fragment_entry)
                .unwrap();
        }

        if properties.depth_test_enable {
            pipeline = pipeline
                .depth_test_enable(properties.depth_test_write_enable, properties.depth_test_op);
        } else {
            pipeline = pipeline.depth_test_disable();
        }

        for (stage, ty, bindings) in &properties.binding_groups {
            pipeline = pipeline.binding_group(*ty);
            for binding in bindings {
                pipeline = pipeline.binding(*binding, *stage);
            }
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
    ) -> PipelineData {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        match &properties {
            PipelineProperties::Compute(properties) => self.load_compute(properties),
            PipelineProperties::Graphics(properties) => self.load_graphics(properties),
        }
    }

    fn unload(&mut self, _resource: Resource<Pipeline>) {}
}
