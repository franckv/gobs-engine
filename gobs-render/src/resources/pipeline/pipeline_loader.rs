use gobs_core::logger;
use gobs_render_hal::{DynamicStateElem, Rect2D, RenderHAL, Viewport};
use gobs_resource::{
    ResourceRegistry, {Resource, ResourceError, ResourceHandle, ResourceLoader, ResourceProperties},
};

use crate::resources::pipeline::{
    GraphicsPipelineProperties, Pipeline, PipelineProperties,
    pipeline::{ComputePipelineProperties, PipelineData},
};

pub struct PipelineLoader {}

impl PipelineLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_compute(
        &self,
        hal: &mut dyn RenderHAL,
        properties: &ComputePipelineProperties,
    ) -> PipelineData {
        let mut pipeline = hal.create_compute_pipeline(&properties.name);

        if let Some(shader) = &properties.compute_shader {
            pipeline = pipeline.shader(shader, &properties.compute_entry);
        }

        for binding_group_layout in &properties.binding_groups {
            pipeline = pipeline.binding_group(binding_group_layout.clone());
        }

        let pipeline = pipeline.build(hal);

        PipelineData { pipeline }
    }

    pub fn load_graphics(
        &self,
        hal: &mut dyn RenderHAL,
        properties: &GraphicsPipelineProperties,
    ) -> PipelineData {
        tracing::debug!(target: logger::RESOURCES, "Loading pipeline: {:?}", properties);

        // let mut pipeline = GfxPipeline::graphics(&properties.name, self.device.clone())

        let mut pipeline = hal
            .create_graphics_pipeline(&properties.name)
            .push_constants(properties.object_data_layout.clone())
            .vertex_attributes(properties.vertex_attributes)
            .viewports(vec![Viewport::new(0., 0., 0., 0.)])
            .scissors(vec![Rect2D::new(0, 0, 0, 0)])
            .dynamic_states(&[DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .polygon_mode(properties.polygon_mode)
            .front_face(properties.front_face)
            .cull_mode(properties.cull_mode)
            .blending_enabled(properties.blend_mode)
            .attachments(properties.color_format, properties.depth_format);

        if let Some(shader) = &properties.vertex_shader
            && let Some(entry) = &properties.vertex_entry
        {
            pipeline = pipeline.vertex_shader(shader, entry);
        }

        if let Some(shader) = &properties.fragment_shader
            && let Some(entry) = &properties.fragment_entry
        {
            pipeline = pipeline.fragment_shader(shader, entry);
        }

        if properties.depth_test_enable {
            pipeline = pipeline
                .depth_test_enable(properties.depth_test_write_enable, properties.depth_test_op);
        } else {
            pipeline = pipeline.depth_test_disable();
        }

        for binding_group_layout in &properties.binding_groups {
            pipeline = pipeline.binding_group(binding_group_layout.clone());
        }

        let pipeline = pipeline.build(hal);

        tracing::debug!(target: logger::RESOURCES, "Loaded pipeline: {:?}", pipeline);

        PipelineData { pipeline }
    }
}

impl Default for PipelineLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLoader<Pipeline> for PipelineLoader {
    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load<'a>(
        &mut self,
        hal: &mut (dyn RenderHAL + 'a),
        handle: &ResourceHandle<Pipeline>,
        registry: &mut ResourceRegistry,
    ) -> Result<PipelineData, ResourceError> {
        let resource = registry.get_mut(handle);
        let properties = &mut resource.properties;

        tracing::debug!(target: logger::RESOURCES, "Load pipeline resource {}", properties.name());

        let data = match &properties {
            PipelineProperties::Compute(properties) => self.load_compute(hal, properties),
            PipelineProperties::Graphics(properties) => self.load_graphics(hal, properties),
        };

        Ok(data)
    }

    fn unload(&mut self, _resource: Resource<Pipeline>) {}

    fn flush(&mut self) {}
}
