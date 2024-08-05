use std::sync::Arc;

use anyhow::Result;
use indexmap::IndexMap;
use parking_lot::RwLock;

use gobs_utils::load;
use gobs_vulkan as vk;

use crate::backend::vulkan::VkBindingGroup;
use crate::{
    backend::vulkan::VkDevice, BindingGroupType, BlendMode, CompareOp, CullMode, DynamicStateElem,
    FrontFace, ImageFormat, Pipeline, PipelineId, PolygonMode, Rect2D, Viewport,
};

pub struct VkPipeline {
    pub(crate) id: PipelineId,
    pub(crate) pipeline: Arc<vk::pipeline::Pipeline>,
    pub(crate) ds_pools: IndexMap<BindingGroupType, RwLock<vk::descriptor::DescriptorSetPool>>,
}

impl Pipeline for VkPipeline {
    fn id(&self) -> PipelineId {
        self.id
    }

    fn graphics(device: &VkDevice) -> VkGraphicsPipelineBuilder {
        VkGraphicsPipelineBuilder::new(device)
    }

    fn compute(device: &VkDevice) -> VkComputePipelineBuilder {
        VkComputePipelineBuilder::new(device)
    }

    fn create_binding_group(self: &Arc<Self>, ty: BindingGroupType) -> Result<VkBindingGroup> {
        if let Some(ds_pool) = self.ds_pools.get(&ty) {
            let ds = ds_pool.write().allocate();

            Ok(VkBindingGroup {
                ds,
                bind_group_type: ty,
                pipeline: self.clone(),
            })
        } else {
            Err(anyhow::Error::msg("DS pool not found"))
        }
    }
    
    fn reset_binding_group(self: &Arc<Self>, ty: BindingGroupType) {
        if let Some(ds_pool) = self.ds_pools.get(&ty) {
            ds_pool.write().reset();
        }
    }
}

pub struct VkGraphicsPipelineBuilder {
    device: Arc<vk::device::Device>,
    builder: vk::pipeline::GraphicsPipelineBuilder,
    current_binding_group: Option<BindingGroupType>,
    current_ds_layout: Option<vk::descriptor::DescriptorSetLayoutBuilder>,
    ds_pools: IndexMap<BindingGroupType, RwLock<vk::descriptor::DescriptorSetPool>>,
    ds_pool_size: usize,
    push_constants: usize,
}

impl VkGraphicsPipelineBuilder {
    fn new(device: &VkDevice) -> Self {
        Self {
            device: device.device.clone(),
            builder: vk::pipeline::Pipeline::graphics_builder(device.device.clone()),
            current_binding_group: None,
            current_ds_layout: None,
            ds_pools: IndexMap::new(),
            ds_pool_size: 10,
            push_constants: 0,
        }
    }

    pub fn vertex_shader(mut self, filename: &str, entry: &str) -> Self {
        let shader_file = load::get_asset_dir(filename, load::AssetType::SHADER).unwrap();
        let shader = vk::pipeline::Shader::from_file(
            shader_file,
            self.device.clone(),
            vk::pipeline::ShaderType::Vertex,
        );

        self.builder = self.builder.vertex_shader(entry, shader);

        self
    }

    pub fn fragment_shader(mut self, filename: &str, entry: &str) -> Self {
        let shader_file = load::get_asset_dir(filename, load::AssetType::SHADER).unwrap();
        let shader = vk::pipeline::Shader::from_file(
            shader_file,
            self.device.clone(),
            vk::pipeline::ShaderType::Fragment,
        );

        self.builder = self.builder.fragment_shader(entry, shader);

        self
    }

    pub fn pool_size(mut self, size: usize) -> Self {
        self.ds_pool_size = size;

        self
    }

    pub fn push_constants(mut self, size: usize) -> Self {
        self.push_constants = size;

        self
    }

    fn save_binding_group(mut self) -> Self {
        if let Some(binding_group) = self.current_binding_group {
            let ds_layout = self.current_ds_layout.unwrap().build(self.device.clone(), false);

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

    pub fn binding_group(mut self, binding_group_type: BindingGroupType) -> Self {
        self = self.save_binding_group();

        self.current_binding_group = Some(binding_group_type);
        self.current_ds_layout = Some(vk::descriptor::DescriptorSetLayout::builder());

        self
    }

    pub fn binding(
        mut self,
        ty: vk::descriptor::DescriptorType,
        stage: vk::descriptor::DescriptorStage,
    ) -> Self {
        let ds_layout = self.current_ds_layout.unwrap().binding(ty, stage);

        self.current_ds_layout = Some(ds_layout);

        self
    }

    pub fn current_binding_group(&self) -> Option<BindingGroupType> {
        self.current_binding_group.clone()
    }

    pub fn polygon_mode(mut self, mode: PolygonMode) -> Self {
        self.builder = self.builder.polygon_mode(mode);

        self
    }

    pub fn viewports(mut self, viewports: Vec<Viewport>) -> Self {
        self.builder = self.builder.viewports(viewports);

        self
    }

    pub fn scissors(mut self, scissors: Vec<Rect2D>) -> Self {
        self.builder = self.builder.scissors(scissors);

        self
    }

    pub fn dynamic_states(mut self, states: &[DynamicStateElem]) -> Self {
        self.builder = self.builder.dynamic_states(states);

        self
    }

    pub fn attachments(
        mut self,
        color_format: Option<ImageFormat>,
        depth_format: Option<ImageFormat>,
    ) -> Self {
        self.builder = self.builder.attachments(color_format, depth_format);

        self
    }

    pub fn depth_test_disable(mut self) -> Self {
        self.builder = self.builder.depth_test_disable();

        self
    }

    pub fn depth_test_enable(mut self, write_enable: bool, op: CompareOp) -> Self {
        self.builder = self.builder.depth_test_enable(write_enable, op);

        self
    }

    pub fn blending_enabled(mut self, blend_mode: BlendMode) -> Self {
        self.builder = self.builder.blending_enabled(blend_mode);

        self
    }

    pub fn cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.builder = self.builder.cull_mode(cull_mode);

        self
    }

    pub fn front_face(mut self, front_face: FrontFace) -> Self {
        self.builder = self.builder.front_face(front_face);

        self
    }

    pub fn build(mut self) -> Arc<VkPipeline> {
        self = self.save_binding_group();

        let ds_pools = self.ds_pools;
        let mut descriptor_layouts = vec![];

        for (_, ds_pool) in &ds_pools {
            descriptor_layouts.push(ds_pool.read().descriptor_layout.clone());
        }

        let pipeline_layout = vk::pipeline::PipelineLayout::new(
            self.device.clone(),
            &descriptor_layouts,
            self.push_constants,
        );

        let pipeline = self.builder.layout(pipeline_layout).build();

        Arc::new(VkPipeline {
            id: PipelineId::new_v4(),
            pipeline,
            ds_pools,
        })
    }
}

pub struct VkComputePipelineBuilder {
    device: Arc<vk::device::Device>,
    builder: vk::pipeline::ComputePipelineBuilder,
    current_binding_group: Option<BindingGroupType>,
    current_ds_layout: Option<vk::descriptor::DescriptorSetLayoutBuilder>,
    ds_pools: IndexMap<BindingGroupType, RwLock<vk::descriptor::DescriptorSetPool>>,
    ds_pool_size: usize,
    push_constants: usize,
}

impl VkComputePipelineBuilder {
    fn new(device: &VkDevice) -> Self {
        Self {
            device: device.device.clone(),
            builder: vk::pipeline::Pipeline::compute_builder(device.device.clone()),
            current_binding_group: None,
            current_ds_layout: None,
            ds_pools: IndexMap::new(),
            ds_pool_size: 10,
            push_constants: 0,
        }
    }

    pub fn shader(mut self, filename: &str, entry: &str) -> Self {
        let compute_file = load::get_asset_dir(filename, load::AssetType::SHADER).unwrap();
        let compute_shader = vk::pipeline::Shader::from_file(
            compute_file,
            self.device.clone(),
            vk::pipeline::ShaderType::Compute,
        );

        self.builder = self.builder.compute_shader(entry, compute_shader);

        self
    }

    fn save_binding_group(mut self) -> Self {
        if let Some(binding_group) = self.current_binding_group {
            let ds_layout = self.current_ds_layout.unwrap().build(self.device.clone(), false);

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

    pub fn binding_group(mut self, binding_group_type: BindingGroupType) -> Self {
        self = self.save_binding_group();

        self.current_binding_group = Some(binding_group_type);
        self.current_ds_layout = Some(vk::descriptor::DescriptorSetLayout::builder());

        self
    }

    pub fn binding(mut self, ty: vk::descriptor::DescriptorType) -> Self {
        let ds_layout = self
            .current_ds_layout
            .unwrap()
            .binding(ty, vk::descriptor::DescriptorStage::Compute);

        self.current_ds_layout = Some(ds_layout);

        self
    }

    pub fn build(mut self) -> Arc<VkPipeline> {
        self = self.save_binding_group();

        let ds_pools = self.ds_pools;
        let mut descriptor_layouts = vec![];

        for (_, ds_pool) in &ds_pools {
            descriptor_layouts.push(ds_pool.read().descriptor_layout.clone());
        }

        let pipeline_layout = vk::pipeline::PipelineLayout::new(
            self.device.clone(),
            &descriptor_layouts,
            self.push_constants,
        );

        let pipeline = self.builder.layout(pipeline_layout).build();

        Arc::new(VkPipeline {
            id: PipelineId::new_v4(),
            pipeline,
            ds_pools,
        })
    }
}
