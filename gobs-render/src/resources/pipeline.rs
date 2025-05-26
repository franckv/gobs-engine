use std::sync::Arc;

use gobs_core::ImageFormat;
use gobs_gfx::{
    BindingGroupType, BlendMode, CompareOp, CullMode, DescriptorStage, DescriptorType, FrontFace,
    GfxPipeline,
};
use gobs_resource::resource::ResourceType;

use crate::resources::pipeline_loader::PipelineLoader;

#[derive(Clone, Copy, Debug)]
pub struct Pipeline;

impl ResourceType for Pipeline {
    type ResourceData = PipelineData;
    type ResourceProperties = PipelineProperties;
    type ResourceParameter = ();
    type ResourceLoader = PipelineLoader;
}

#[derive(Clone)]
pub struct PipelineData {
    pub pipeline: Arc<GfxPipeline>,
}

#[derive(Clone)]
pub enum PipelineProperties {
    Compute(ComputePipelineProperties),
    Graphics(GraphicsPipelineProperties),
}

impl PipelineProperties {
    pub fn compute(name: &str) -> ComputePipelineProperties {
        ComputePipelineProperties::new(name)
    }

    pub fn graphics(name: &str) -> GraphicsPipelineProperties {
        GraphicsPipelineProperties::new(name)
    }
}

#[derive(Clone, Debug)]
pub struct GraphicsPipelineProperties {
    pub name: String,
    pub(crate) vertex_entry: String,
    pub(crate) vertex_shader: Option<String>,
    pub(crate) fragment_entry: String,
    pub(crate) fragment_shader: Option<String>,
    pub(crate) binding_groups: Vec<(DescriptorStage, BindingGroupType, Vec<DescriptorType>)>,
    pub(crate) last_binding_group: BindingGroupType,
    pub(crate) ds_pool_size: usize,
    pub(crate) push_constants: usize,
    pub(crate) color_format: Option<ImageFormat>,
    pub(crate) depth_format: Option<ImageFormat>,
    pub(crate) depth_test_enable: bool,
    pub(crate) depth_test_write_enable: bool,
    pub(crate) depth_test_op: CompareOp,
    pub(crate) front_face: FrontFace,
    pub(crate) cull_mode: CullMode,
    pub(crate) blend_mode: BlendMode,
}

impl GraphicsPipelineProperties {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vertex_entry: "main".to_string(),
            vertex_shader: None,
            fragment_entry: "main".to_string(),
            fragment_shader: None,
            binding_groups: Vec::new(),
            last_binding_group: BindingGroupType::None,
            ds_pool_size: 10,
            push_constants: 0,
            color_format: None,
            depth_format: None,
            depth_test_enable: true,
            depth_test_write_enable: true,
            depth_test_op: CompareOp::Never,
            front_face: FrontFace::default(),
            cull_mode: CullMode::default(),
            blend_mode: BlendMode::None,
        }
    }

    pub fn vertex_entry(mut self, entry: &str) -> Self {
        self.vertex_entry = entry.to_string();

        self
    }

    pub fn vertex_shader(mut self, shader: &str) -> Self {
        self.vertex_shader = Some(shader.to_string());

        self
    }

    pub fn fragment_entry(mut self, entry: &str) -> Self {
        self.fragment_entry = entry.to_string();

        self
    }

    pub fn fragment_shader(mut self, shader: &str) -> Self {
        self.fragment_shader = Some(shader.to_string());

        self
    }

    pub fn color_format(mut self, format: ImageFormat) -> Self {
        self.color_format = Some(format);

        self
    }

    pub fn depth_format(mut self, format: ImageFormat) -> Self {
        self.depth_format = Some(format);

        self
    }

    pub fn depth_test_disable(mut self) -> Self {
        self.depth_test_enable = false;

        self
    }

    pub fn depth_test_enable(mut self, write_enable: bool, op: CompareOp) -> Self {
        self.depth_test_enable = true;
        self.depth_test_write_enable = write_enable;
        self.depth_test_op = op;

        self
    }

    pub fn front_face(mut self, front_face: FrontFace) -> Self {
        self.front_face = front_face;

        self
    }

    pub fn cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.cull_mode = cull_mode;

        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;

        self
    }

    pub fn binding_group(mut self, stage: DescriptorStage, ty: BindingGroupType) -> Self {
        self.binding_groups.push((stage, ty, Vec::new()));
        self.last_binding_group = ty;

        self
    }

    pub fn binding(mut self, ty: DescriptorType) -> Self {
        if let Some((_, _, group)) = self.binding_groups.last_mut() {
            group.push(ty);
        }

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

    pub fn wrap(self) -> PipelineProperties {
        PipelineProperties::Graphics(self)
    }
}

#[derive(Clone)]
pub struct ComputePipelineProperties {
    pub name: String,
    pub(crate) compute_entry: String,
    pub(crate) compute_shader: Option<String>,
    pub(crate) binding_groups: Vec<(DescriptorStage, Vec<DescriptorType>)>,
}

impl ComputePipelineProperties {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            compute_entry: "main".to_string(),
            compute_shader: None,
            binding_groups: Vec::new(),
        }
    }

    pub fn compute_entry(mut self, entry: &str) -> Self {
        self.compute_entry = entry.to_string();

        self
    }

    pub fn compute_shader(mut self, shader: &str) -> Self {
        self.compute_shader = Some(shader.to_string());

        self
    }

    pub fn binding_group(mut self, stage: DescriptorStage) -> Self {
        self.binding_groups.push((stage, Vec::new()));

        self
    }

    pub fn binding(mut self, ty: DescriptorType) -> Self {
        if let Some((_, group)) = self.binding_groups.last_mut() {
            group.push(ty);
        }

        self
    }

    pub fn wrap(self) -> PipelineProperties {
        PipelineProperties::Compute(self)
    }
}
