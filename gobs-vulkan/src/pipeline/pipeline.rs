use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use log::trace;

use crate::descriptor::DescriptorSetLayout;
use crate::device::Device;
use crate::pipeline::{Shader, ShaderType, VertexLayout, PipelineLayout};
use crate::renderpass::RenderPass;
use crate::Wrap;

pub struct Viewport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    min_depth: f32,
    max_depth: f32,
}

impl Viewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Viewport {
            x,
            y,
            width,
            height,
            min_depth: 0.,
            max_depth: 1.,
        }
    }

    fn raw(&self) -> vk::Viewport {
        vk::Viewport {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            min_depth: self.min_depth,
            max_depth: self.max_depth,
        }
    }
}

pub struct Rect2D {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Rect2D {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Rect2D {
            x, 
            y,
            width,
            height
        }
    }

    fn raw(&self) -> vk::Rect2D {
        vk::Rect2D {
            offset: vk::Offset2D { x: self.x, y: self.y },
            extent: vk::Extent2D { width: self.width, height: self.height },
        }
    }
}

struct ViewportState {
    viewports: Vec<vk::Viewport>,
    scissors: Vec<vk::Rect2D>
}

impl ViewportState {
    fn new(viewports: &Vec<Viewport>, scissors: &Vec<Rect2D>) -> Self {
        ViewportState {
            viewports: viewports.iter().map(|v| v.raw()).collect::<Vec<vk::Viewport>>(),
            scissors: scissors.iter().map(|s| s.raw()).collect::<Vec<vk::Rect2D>>()
        }
    }

    fn info(&self) -> vk::PipelineViewportStateCreateInfoBuilder {
        vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&self.scissors)
            .viewports(&self.viewports)
    }
}

pub enum DynamicStateElem {
    Viewport,
    Scissor
}

impl DynamicStateElem {
    fn raw(&self) -> vk::DynamicState {
        match self {
            DynamicStateElem::Viewport => vk::DynamicState::VIEWPORT,
            DynamicStateElem::Scissor => vk::DynamicState::SCISSOR,
        }
    }
}

struct DynamicStates {
    dynamic_states: Vec<vk::DynamicState>
}

impl DynamicStates {
    fn new(states: &Vec<DynamicStateElem>) -> Self {
        DynamicStates {
            dynamic_states: states.iter().map(|s| s.raw()).collect::<Vec<vk::DynamicState>>(),
        }
    }

    fn info(&self) -> vk::PipelineDynamicStateCreateInfoBuilder {
        vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&self.dynamic_states)
    }
}

/// primitive topology
struct InputAssemblyState;

impl InputAssemblyState {
    fn new() -> Self {
        InputAssemblyState
    }

    fn info(&self) -> vk::PipelineInputAssemblyStateCreateInfoBuilder {
        vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    }
}

struct RasterizationState;

impl RasterizationState {
    fn new() -> Self {
        RasterizationState
    }

    fn info(&self) -> vk::PipelineRasterizationStateCreateInfoBuilder {
        vk::PipelineRasterizationStateCreateInfo::builder()
        .line_width(1.)
        .front_face(vk::FrontFace::CLOCKWISE)
        .cull_mode(vk::CullModeFlags::NONE)
        .polygon_mode(vk::PolygonMode::FILL)
    }
}

struct MultisampleState;

impl MultisampleState {
    fn new() -> Self {
        MultisampleState
    }

    fn info(&self) -> vk::PipelineMultisampleStateCreateInfoBuilder {
        vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
    }
}

pub struct ShaderStage {
    entry: CString,
    shader: Shader
}

impl ShaderStage {
    fn new(entry: &str, shader: Shader) -> Self {
        ShaderStage {
            entry: CString::new(entry).unwrap(),
            shader
        }
    }

    fn info(&self) -> vk::PipelineShaderStageCreateInfoBuilder {
        vk::PipelineShaderStageCreateInfo::builder()
        .stage(match self.shader.ty {
            ShaderType::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
        })
        .module(self.shader.raw())
        .name(&self.entry)
    }
}

struct StencilOpState;

impl StencilOpState {
    fn new() -> Self {
        StencilOpState
    }

    fn info(&self) -> vk::StencilOpState {
        vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        }    
    }
}

struct DepthStencilState {
    op_state: vk::StencilOpState
}

impl DepthStencilState {
    fn new(op_state: StencilOpState) -> Self {
        DepthStencilState {
            op_state: op_state.info()
        }
    }

    fn info(&self) -> vk::PipelineDepthStencilStateCreateInfoBuilder {
        vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.)
            .max_depth_bounds(1.)
            .stencil_test_enable(false)
            .front(self.op_state)
            .back(self.op_state)
    }
}

struct ColorBlendAttachmentState;

impl ColorBlendAttachmentState {
    fn new() -> Self {
        ColorBlendAttachmentState
    }

    fn info(&self) -> vk::PipelineColorBlendAttachmentStateBuilder {
        vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::all())
    }
}

struct ColorBlendState {
    attachment_state: Vec<vk::PipelineColorBlendAttachmentState>
}

impl ColorBlendState {
    fn new(attachment_state: ColorBlendAttachmentState) -> Self {
        ColorBlendState {
            attachment_state: vec![attachment_state.info().build()]
        }
    }

    fn info(&self) -> vk::PipelineColorBlendStateCreateInfoBuilder {
        vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::COPY)
            .attachments(&self.attachment_state)
    }
}

struct VertexInputState {
    binding_desc: Vec<vk::VertexInputBindingDescription>,
    attribute_desc: Vec<vk::VertexInputAttributeDescription>
}

impl VertexInputState {
    fn new(vertex_layout: &VertexLayout) -> Self {
        VertexInputState {
            binding_desc: vertex_layout.binding_description(),
            attribute_desc: vertex_layout.attribute_description()
        }
    }

    fn info(&self) -> vk::PipelineVertexInputStateCreateInfoBuilder {

        vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&self.binding_desc)
        .vertex_attribute_descriptions(&self.attribute_desc)
    }
}

#[derive(Default)]
pub struct PipelineBuilder {
    device: Option<Arc<Device>>,
    renderpass: Option<Arc<RenderPass>>,
    vertex_stage: Option<ShaderStage>,
    fragment_stage: Option<ShaderStage>,
    vertex_input_state: Option<VertexInputState>,
    viewports: Option<Vec<Viewport>>,
    scissors: Option<Vec<Rect2D>>,
    dynamic_states: Option<DynamicStates>,
    pipeline_layout: Option<PipelineLayout>,

}

impl PipelineBuilder {
    pub fn new(device: Arc<Device>) -> Self {
        PipelineBuilder {
            device: Some(device),
            ..Default::default()
        }
    }

    pub fn renderpass(mut self, renderpass: Arc<RenderPass>) -> Self {
        self.renderpass = Some(renderpass);

        self
    }

    pub fn vertex_shader(mut self, entry: &str, vshader: Shader) -> Self {
        self.vertex_stage = Some(ShaderStage::new(entry, vshader));

        self
    }

    pub fn fragment_shader(mut self, entry: &str, fshader: Shader) -> Self {
        self.fragment_stage = Some(ShaderStage::new(entry, fshader));

        self
    }

    pub fn vertex_layout(mut self, vertex_layout: &VertexLayout) -> Self {
        self.vertex_input_state = Some(VertexInputState::new(vertex_layout));

        self
    }

    pub fn viewports(mut self, viewports: Vec<Viewport>) -> Self {
        self.viewports = Some(viewports);

        self
    }

    pub fn scissors(mut self, scissors: Vec<Rect2D>) -> Self {
        self.scissors = Some(scissors);

        self
    }

    pub fn dynamic_states(mut self, states: &Vec<DynamicStateElem>) -> Self {
        self.dynamic_states = Some(DynamicStates::new(states));

        self
    }

    pub fn descriptor_layout(mut self, descriptor_layout: Arc<DescriptorSetLayout>) -> Self {
        self.pipeline_layout = Some(PipelineLayout::new(self.device.clone().unwrap(), descriptor_layout));

        self
    }

    pub fn build(self) -> Pipeline {
        let device = self.device.unwrap();

        let vertex_stage = self.vertex_stage.unwrap();
        let vertex_stage_info = vertex_stage.info();

        let fragment_stage = self.fragment_stage.unwrap();
        let fragment_stage_info = fragment_stage.info();

        let shader_stages = [vertex_stage_info.build(), fragment_stage_info.build()];

        let vertex_input_state = self.vertex_input_state.unwrap();
        let vertex_input_state_info = vertex_input_state.info();       

        let viewports = self.viewports.unwrap();
        let scissors = self.scissors.unwrap();

        let viewport_state = ViewportState::new(&viewports, &scissors);
        let viewport_state_info = viewport_state.info();

        let dynamic_states = self.dynamic_states.unwrap();
        let dynamic_states_info = dynamic_states.info();

        let input_assembly_state = InputAssemblyState::new();
        let input_assembly_state_info = input_assembly_state.info();

        let rasterization_state = RasterizationState::new();
        let rasterization_state_info = rasterization_state.info();
        
        let multisample_state = MultisampleState::new();
        let multisample_state_info = multisample_state.info();

        let op_state = StencilOpState::new();
        let depth_stencil_state = DepthStencilState::new(op_state);
        let depth_stencil_state_info = depth_stencil_state.info();

        let color_blend_attachment = ColorBlendAttachmentState::new();
        let color_blend_state = ColorBlendState::new(color_blend_attachment);
        let color_blend_state_info = color_blend_state.info();

        let pipeline_layout = self.pipeline_layout.unwrap();

        let renderpass = self.renderpass.unwrap();

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_info)
            .viewport_state(&viewport_state_info)
            .dynamic_state(&dynamic_states_info)
            .input_assembly_state(&input_assembly_state_info)
            .rasterization_state(&rasterization_state_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_stencil_state_info)
            .color_blend_state(&color_blend_state_info)
            .layout(pipeline_layout.raw())
            .render_pass(renderpass.raw());

        let pipeline = unsafe {
            device.raw().create_graphics_pipelines(vk::PipelineCache::null(),
                                                   &[pipeline_info.build()], None).unwrap()[0]
        };

        Pipeline {
            device: device,
            _renderpass: renderpass,
            layout: pipeline_layout,
            pipeline
        }
    }
}

pub struct Pipeline {
    device: Arc<Device>,
    _renderpass: Arc<RenderPass>,
    pub(crate) layout: PipelineLayout,
    
    pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn builder(device: Arc<Device>) -> PipelineBuilder {
        PipelineBuilder::new(device)
    }

    /*
    pub fn new(device: Arc<Device>,
            vshader: Shader, fshader: Shader,
            vertex_layout: VertexLayout,
            descriptor_layout: Arc<DescriptorSetLayout>,
            renderpass: Arc<RenderPass>,
            _subpass: u32) -> Self {

            Self::builder(device)
            .renderpass(renderpass)
            .vertex_shader("main", vshader)
            .fragment_shader("main", fshader)
            .vertex_layout(&vertex_layout)
            .viewports(vec![Viewport::new(0., 0., WIDTH as f32, HEIGHT as f32)])
            .scissors(vec![Rect2D::new(0, 0, WIDTH, HEIGHT)])
            .dynamic_states(&vec![DynamicStateElem::Viewport, DynamicStateElem::Scissor])
            .descriptor_layout(descriptor_layout)
            .build()
    }
    */
}

impl Wrap<vk::Pipeline> for Pipeline {
    fn raw(&self) -> vk::Pipeline {
        self.pipeline
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        trace!("Drop pipeline");
        unsafe {
            self.device.raw().destroy_pipeline(self.pipeline, None);
        }
    }
}
