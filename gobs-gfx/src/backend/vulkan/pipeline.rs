use std::sync::Arc;

use indexmap::IndexMap;
use parking_lot::RwLock;

use gobs_core::{ImageFormat, logger};
use gobs_resource::geometry::VertexAttribute;
use gobs_resource::load;
use gobs_vulkan as vk;

use crate::GfxError;
use crate::backend::vulkan::{bindgroup::VkBindingGroup, device::VkDevice, renderer::VkRenderer};
use crate::{
    BindingGroupType, BlendMode, CompareOp, ComputePipelineBuilder, CullMode, DynamicStateElem,
    FrontFace, GraphicsPipelineBuilder, Pipeline, PipelineId, PolygonMode, Rect2D, Viewport,
};

#[derive(Debug)]
pub struct VkPipeline {
    pub(crate) name: String,
    pub(crate) id: PipelineId,
    // TODO: handle compute shaders attributes
    pub(crate) vertex_attributes: VertexAttribute,
    pub(crate) pipeline: Arc<vk::pipelines::Pipeline>,
    pub(crate) ds_pools: IndexMap<BindingGroupType, RwLock<vk::descriptor::DescriptorSetPool>>,
}

impl Pipeline<VkRenderer> for VkPipeline {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> PipelineId {
        self.id
    }

    fn vertex_attributes(&self) -> VertexAttribute {
        self.vertex_attributes
    }

    fn graphics(name: &str, device: &VkDevice) -> VkGraphicsPipelineBuilder {
        VkGraphicsPipelineBuilder::new(name, device)
    }

    fn compute(name: &str, device: &VkDevice) -> VkComputePipelineBuilder {
        VkComputePipelineBuilder::new(name, device)
    }

    fn create_binding_group(
        self: &Arc<Self>,
        ty: BindingGroupType,
    ) -> Result<VkBindingGroup, GfxError> {
        let ds = self
            .ds_pools
            .get(&ty)
            .ok_or(GfxError::DsPoolCreation)?
            .write()
            .allocate();

        Ok(VkBindingGroup {
            ds,
            bind_group_type: ty,
            pipeline: self.clone(),
        })
    }

    fn reset_binding_group(self: &Arc<Self>, ty: BindingGroupType) {
        if let Some(ds_pool) = self.ds_pools.get(&ty) {
            ds_pool.write().reset();
        }
    }
}

pub struct VkGraphicsPipelineBuilder {
    name: String,
    device: Arc<vk::device::Device>,
    builder: vk::pipelines::GraphicsPipelineBuilder,
    current_binding_group: Option<BindingGroupType>,
    current_ds_layout: Option<vk::descriptor::DescriptorSetLayoutBuilder>,
    ds_pools: IndexMap<BindingGroupType, RwLock<vk::descriptor::DescriptorSetPool>>,
    ds_pool_size: usize,
    push_constants: usize,
    vertex_attributes: VertexAttribute,
}

impl GraphicsPipelineBuilder<VkRenderer> for VkGraphicsPipelineBuilder {
    fn vertex_shader(mut self, filename: &str, entry: &str) -> Result<Self, GfxError> {
        let shader_file = load::get_asset_dir(filename, load::AssetType::SHADER)?;
        let shader = vk::pipelines::Shader::from_file(
            shader_file,
            self.device.clone(),
            vk::pipelines::ShaderType::Vertex,
        )?;

        self.builder = self.builder.vertex_shader(entry, shader);

        Ok(self)
    }

    fn fragment_shader(mut self, filename: &str, entry: &str) -> Result<Self, GfxError> {
        let shader_file = load::get_asset_dir(filename, load::AssetType::SHADER)?;
        let shader = vk::pipelines::Shader::from_file(
            shader_file,
            self.device.clone(),
            vk::pipelines::ShaderType::Fragment,
        )?;

        self.builder = self.builder.fragment_shader(entry, shader);

        Ok(self)
    }

    fn pool_size(mut self, size: usize) -> Self {
        self.ds_pool_size = size;

        self
    }

    fn push_constants(mut self, size: usize) -> Self {
        self.push_constants = size;

        self
    }

    fn vertex_attributes(mut self, vertex_attributes: VertexAttribute) -> Self {
        self.vertex_attributes = vertex_attributes;

        self
    }

    fn binding_group(mut self, binding_group_type: BindingGroupType) -> Self {
        self = self.save_binding_group();

        let set = binding_group_type.set();
        self.current_binding_group = Some(binding_group_type);
        self.current_ds_layout = Some(vk::descriptor::DescriptorSetLayout::builder(set));

        self
    }

    fn current_binding_group(&self) -> Option<BindingGroupType> {
        self.current_binding_group
    }

    fn binding(
        mut self,
        ty: vk::descriptor::DescriptorType,
        stage: vk::descriptor::DescriptorStage,
    ) -> Self {
        let ds_layout = self.current_ds_layout.unwrap().binding(ty, stage);

        self.current_ds_layout = Some(ds_layout);

        self
    }

    fn polygon_mode(mut self, mode: PolygonMode) -> Self {
        self.builder = self.builder.polygon_mode(mode);

        self
    }

    fn viewports(mut self, viewports: Vec<Viewport>) -> Self {
        self.builder = self.builder.viewports(viewports);

        self
    }

    fn scissors(mut self, scissors: Vec<Rect2D>) -> Self {
        self.builder = self.builder.scissors(scissors);

        self
    }

    fn dynamic_states(mut self, states: &[DynamicStateElem]) -> Self {
        self.builder = self.builder.dynamic_states(states);

        self
    }

    fn attachments(
        mut self,
        color_format: Option<ImageFormat>,
        depth_format: Option<ImageFormat>,
    ) -> Self {
        self.builder = self.builder.attachments(color_format, depth_format);

        self
    }

    fn depth_test_disable(mut self) -> Self {
        self.builder = self.builder.depth_test_disable();

        self
    }

    fn depth_test_enable(mut self, write_enable: bool, op: CompareOp) -> Self {
        self.builder = self.builder.depth_test_enable(write_enable, op);

        self
    }

    fn blending_enabled(mut self, blend_mode: BlendMode) -> Self {
        self.builder = self.builder.blending_enabled(blend_mode);

        self
    }

    fn cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.builder = self.builder.cull_mode(cull_mode);

        self
    }

    fn front_face(mut self, front_face: FrontFace) -> Self {
        self.builder = self.builder.front_face(front_face);

        self
    }

    fn build(mut self) -> Arc<VkPipeline> {
        tracing::debug!(target: logger::RENDER, "Creating pipeline: {}", self.name);

        self = self.save_binding_group();

        let ds_pools = self.ds_pools;
        let mut descriptor_layouts = vec![];

        for (_, ds_pool) in &ds_pools {
            descriptor_layouts.push(ds_pool.read().descriptor_layout.clone());
        }

        let pipeline_layout = vk::pipelines::PipelineLayout::new(
            self.device.clone(),
            descriptor_layouts,
            self.push_constants,
        );

        let pipeline = self.builder.layout(pipeline_layout).build();

        Arc::new(VkPipeline {
            name: self.name,
            id: PipelineId::new_v4(),
            vertex_attributes: self.vertex_attributes,
            pipeline,
            ds_pools,
        })
    }
}

impl VkGraphicsPipelineBuilder {
    fn new(name: &str, device: &VkDevice) -> Self {
        Self {
            name: name.to_string(),
            device: device.device.clone(),
            builder: vk::pipelines::Pipeline::graphics_builder(device.device.clone()),
            current_binding_group: None,
            current_ds_layout: None,
            ds_pools: IndexMap::new(),
            ds_pool_size: 10,
            push_constants: 0,
            vertex_attributes: VertexAttribute::empty(),
        }
    }

    fn save_binding_group(mut self) -> Self {
        if let Some(binding_group) = self.current_binding_group {
            let push = binding_group == BindingGroupType::SceneData;
            let ds_layout = self
                .current_ds_layout
                .unwrap()
                .build(self.device.clone(), push);

            let ds_pool = vk::descriptor::DescriptorSetPool::new(
                self.device.clone(),
                ds_layout,
                self.ds_pool_size,
            );

            self.ds_pools.insert(binding_group, RwLock::new(ds_pool));
        }

        self.current_binding_group = None;
        self.current_ds_layout = None;

        self
    }
}

pub struct VkComputePipelineBuilder {
    name: String,
    device: Arc<vk::device::Device>,
    builder: vk::pipelines::ComputePipelineBuilder,
    current_binding_group: Option<BindingGroupType>,
    current_ds_layout: Option<vk::descriptor::DescriptorSetLayoutBuilder>,
    ds_pools: IndexMap<BindingGroupType, RwLock<vk::descriptor::DescriptorSetPool>>,
    ds_pool_size: usize,
    push_constants: usize,
}

impl ComputePipelineBuilder<VkRenderer> for VkComputePipelineBuilder {
    fn shader(mut self, filename: &str, entry: &str) -> Result<Self, GfxError> {
        let compute_file = load::get_asset_dir(filename, load::AssetType::SHADER)?;
        let compute_shader = vk::pipelines::Shader::from_file(
            compute_file,
            self.device.clone(),
            vk::pipelines::ShaderType::Compute,
        )?;

        self.builder = self.builder.compute_shader(entry, compute_shader);

        Ok(self)
    }

    fn binding_group(mut self, binding_group_type: BindingGroupType) -> Self {
        self = self.save_binding_group();

        let set = binding_group_type.set();
        self.current_binding_group = Some(binding_group_type);
        self.current_ds_layout = Some(vk::descriptor::DescriptorSetLayout::builder(set));

        self
    }

    fn binding(mut self, ty: vk::descriptor::DescriptorType) -> Self {
        let ds_layout = self
            .current_ds_layout
            .unwrap()
            .binding(ty, vk::descriptor::DescriptorStage::Compute);

        self.current_ds_layout = Some(ds_layout);

        self
    }

    fn build(mut self) -> Arc<VkPipeline> {
        self = self.save_binding_group();

        let ds_pools = self.ds_pools;
        let mut descriptor_layouts = vec![];

        for (_, ds_pool) in &ds_pools {
            descriptor_layouts.push(ds_pool.read().descriptor_layout.clone());
        }

        let pipeline_layout = vk::pipelines::PipelineLayout::new(
            self.device.clone(),
            descriptor_layouts,
            self.push_constants,
        );

        let pipeline = self.builder.layout(pipeline_layout).build();

        Arc::new(VkPipeline {
            name: self.name,
            id: PipelineId::new_v4(),
            pipeline,
            ds_pools,
            vertex_attributes: VertexAttribute::empty(),
        })
    }
}

impl VkComputePipelineBuilder {
    fn new(name: &str, device: &VkDevice) -> Self {
        Self {
            name: name.to_string(),
            device: device.device.clone(),
            builder: vk::pipelines::Pipeline::compute_builder(device.device.clone()),
            current_binding_group: None,
            current_ds_layout: None,
            ds_pools: IndexMap::new(),
            ds_pool_size: 10,
            push_constants: 0,
        }
    }

    fn save_binding_group(mut self) -> Self {
        if let Some(binding_group) = self.current_binding_group {
            let ds_layout = self
                .current_ds_layout
                .unwrap()
                .build(self.device.clone(), false);

            let ds_pool = vk::descriptor::DescriptorSetPool::new(
                self.device.clone(),
                ds_layout,
                self.ds_pool_size,
            );

            self.ds_pools.insert(binding_group, RwLock::new(ds_pool));
        }

        self.current_binding_group = None;
        self.current_ds_layout = None;

        self
    }
}
