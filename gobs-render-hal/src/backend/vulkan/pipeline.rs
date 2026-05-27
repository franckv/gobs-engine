use std::any::Any;
use std::sync::Arc;

use gobs_core::{ImageFormat, logger};
use gobs_resource::load;
use gobs_vulkan as vk;

use crate::{
    Handle, RenderHAL, VertexAttribute,
    backend::{VulkanHAL, VulkanHALExt},
    bindings::BindingGroupLayout,
    pipeline::{ComputePipelineBuilder, GraphicsPipelineBuilder},
};

pub(crate) struct VkComputePipelineBuilder {
    device: Arc<vk::Device>,
    builder: vk::ComputePipelineBuilder,
    descriptor_layouts: Vec<BindingGroupLayout>,
    push_constants: usize,
}

impl ComputePipelineBuilder for VkComputePipelineBuilder {
    fn shader(mut self: Box<Self>, filename: &str, entry: &str) -> Box<dyn ComputePipelineBuilder> {
        let compute_file = load::get_asset_dir(filename, load::AssetType::SHADER).unwrap();
        let compute_shader = vk::pipelines::Shader::from_file(
            compute_file,
            self.device.clone(),
            vk::pipelines::ShaderType::Compute,
        )
        .unwrap();

        self.builder = self.builder.compute_shader(entry, compute_shader);

        self
    }

    fn binding_group(
        mut self: Box<Self>,
        layout: BindingGroupLayout,
    ) -> Box<dyn ComputePipelineBuilder> {
        self.descriptor_layouts.push(layout);

        self
    }

    fn build(self: Box<Self>, hal: &mut dyn RenderHAL) -> Handle {
        let descriptor_layouts = self
            .descriptor_layouts
            .iter()
            .map(|layout| vk_layout(self.device.clone(), layout))
            .collect();

        let pipeline_layout = vk::pipelines::PipelineLayout::new(
            self.device.clone(),
            descriptor_layouts,
            self.push_constants,
        );

        let pipeline = self.builder.layout(pipeline_layout).build();

        let hal = hal.get_mut();

        hal.registry.pipelines.insert(pipeline)
    }
}

impl VkComputePipelineBuilder {
    pub(crate) fn new(name: &str, device: Arc<vk::Device>) -> Self {
        Self {
            device: device.clone(),
            builder: vk::pipelines::Pipeline::compute_builder(name, device.clone()),
            descriptor_layouts: Vec::new(),
            push_constants: 0,
        }
    }
}

pub(crate) struct VkGraphicsPipelineBuilder {
    device: Arc<vk::Device>,
    builder: vk::GraphicsPipelineBuilder,
    descriptor_layouts: Vec<BindingGroupLayout>,
    push_constants: usize,
    vertex_attributes: VertexAttribute,
}

impl GraphicsPipelineBuilder for VkGraphicsPipelineBuilder {
    fn vertex_shader(
        mut self: Box<Self>,
        filename: &str,
        entry: &str,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        let shader_file = load::get_asset_dir(filename, load::AssetType::SHADER).unwrap();

        let shader = vk::pipelines::Shader::from_file(
            shader_file,
            self.device.clone(),
            vk::pipelines::ShaderType::Vertex,
        )
        .unwrap();

        self.builder = self.builder.vertex_shader(entry, shader);

        self
    }

    fn fragment_shader(
        mut self: Box<Self>,
        filename: &str,
        entry: &str,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        let shader_file = load::get_asset_dir(filename, load::AssetType::SHADER).unwrap();

        let shader = vk::pipelines::Shader::from_file(
            shader_file,
            self.device.clone(),
            vk::pipelines::ShaderType::Fragment,
        )
        .unwrap();

        self.builder = self.builder.fragment_shader(entry, shader);

        self
    }

    fn push_constants(mut self: Box<Self>, size: usize) -> Box<dyn GraphicsPipelineBuilder> {
        self.push_constants = size;

        self
    }

    fn vertex_attributes(
        mut self: Box<Self>,
        vertex_attributes: VertexAttribute,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.vertex_attributes = vertex_attributes;

        self
    }

    fn binding_group(
        mut self: Box<Self>,
        layout: BindingGroupLayout,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.descriptor_layouts.push(layout);

        self
    }

    fn polygon_mode(
        mut self: Box<Self>,
        mode: vk::PolygonMode,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.polygon_mode(mode);

        self
    }

    fn viewports(
        mut self: Box<Self>,
        viewports: Vec<vk::Viewport>,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.viewports(viewports);

        self
    }

    fn scissors(
        mut self: Box<Self>,
        scissors: Vec<vk::Rect2D>,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.scissors(scissors);

        self
    }

    fn dynamic_states(
        mut self: Box<Self>,
        states: &[vk::DynamicStateElem],
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.dynamic_states(states);

        self
    }

    fn attachments(
        mut self: Box<Self>,
        color_format: Option<ImageFormat>,
        depth_format: Option<ImageFormat>,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.attachments(color_format, depth_format);

        self
    }

    fn depth_test_disable(mut self: Box<Self>) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.depth_test_disable();

        self
    }

    fn depth_test_enable(
        mut self: Box<Self>,
        write_enable: bool,
        op: vk::CompareOp,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.depth_test_enable(write_enable, op);

        self
    }

    fn blending_enabled(
        mut self: Box<Self>,
        blend_mode: vk::BlendMode,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.blending_enabled(blend_mode);

        self
    }

    fn cull_mode(mut self: Box<Self>, cull_mode: vk::CullMode) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.cull_mode(cull_mode);

        self
    }

    fn front_face(
        mut self: Box<Self>,
        front_face: vk::FrontFace,
    ) -> Box<dyn GraphicsPipelineBuilder> {
        self.builder = self.builder.front_face(front_face);

        self
    }

    fn build(self: Box<Self>, hal: &mut dyn RenderHAL) -> Handle {
        let descriptor_layouts = self
            .descriptor_layouts
            .iter()
            .map(|layout| vk_layout(self.device.clone(), layout))
            .collect();

        let pipeline_layout = vk::pipelines::PipelineLayout::new(
            self.device.clone(),
            descriptor_layouts,
            self.push_constants,
        );

        let pipeline = self.builder.layout(pipeline_layout).build();

        let hal = hal.get_mut();

        hal.registry.pipelines.insert(pipeline)
    }
}

impl VkGraphicsPipelineBuilder {
    pub(crate) fn new(name: &str, device: Arc<vk::Device>) -> Self {
        Self {
            device: device.clone(),
            builder: vk::pipelines::Pipeline::graphics_builder(name, device.clone()),
            descriptor_layouts: Vec::new(),
            push_constants: 0,
            vertex_attributes: VertexAttribute::empty(),
        }
    }
}

fn vk_layout(
    device: Arc<vk::Device>,
    layout: &BindingGroupLayout,
) -> Arc<vk::descriptor::DescriptorSetLayout> {
    let mut ds_layout =
        vk::descriptor::DescriptorSetLayout::builder(layout.binding_group_type.set());

    for (ty, stage) in &layout.bindings {
        ds_layout = ds_layout.binding(*ty, *stage);
    }

    ds_layout.build(device.clone(), layout.binding_group_type.is_push())
}
