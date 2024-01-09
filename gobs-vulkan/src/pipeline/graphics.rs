use std::sync::Arc;

use ash::vk;

use crate::image::ImageFormat;
use crate::pipeline::{Pipeline, PipelineLayout, Rect2D, Shader, ShaderStage, VertexLayout};
use crate::{device::Device, Wrap};

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

struct ViewportState {
    viewports: Vec<vk::Viewport>,
    scissors: Vec<vk::Rect2D>,
}

impl ViewportState {
    fn new(viewports: &Vec<Viewport>, scissors: &Vec<Rect2D>) -> Self {
        ViewportState {
            viewports: viewports
                .iter()
                .map(|v| v.raw())
                .collect::<Vec<vk::Viewport>>(),
            scissors: scissors
                .iter()
                .map(|s| s.raw())
                .collect::<Vec<vk::Rect2D>>(),
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
    Scissor,
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
    dynamic_states: Vec<vk::DynamicState>,
}

impl DynamicStates {
    fn new(states: &Vec<DynamicStateElem>) -> Self {
        DynamicStates {
            dynamic_states: states
                .iter()
                .map(|s| s.raw())
                .collect::<Vec<vk::DynamicState>>(),
        }
    }

    fn info(&self) -> vk::PipelineDynamicStateCreateInfoBuilder {
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&self.dynamic_states)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveTopology {
    Point,
    Triangle,
    Line,
}

impl Default for PrimitiveTopology {
    fn default() -> Self {
        PrimitiveTopology::Triangle
    }
}

impl Into<vk::PrimitiveTopology> for PrimitiveTopology {
    fn into(self) -> vk::PrimitiveTopology {
        match self {
            PrimitiveTopology::Point => vk::PrimitiveTopology::POINT_LIST,
            PrimitiveTopology::Triangle => vk::PrimitiveTopology::TRIANGLE_LIST,
            PrimitiveTopology::Line => vk::PrimitiveTopology::LINE_LIST,
        }
    }
}

/// primitive topology
struct InputAssemblyState {
    primitive_topology: PrimitiveTopology,
}

impl InputAssemblyState {
    fn new(primitive_topology: PrimitiveTopology) -> Self {
        InputAssemblyState { primitive_topology }
    }

    fn info(&self) -> vk::PipelineInputAssemblyStateCreateInfoBuilder {
        vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(self.primitive_topology.into())
            .primitive_restart_enable(false)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

impl Default for PolygonMode {
    fn default() -> Self {
        Self::Fill
    }
}

impl Into<vk::PolygonMode> for PolygonMode {
    fn into(self) -> vk::PolygonMode {
        match self {
            PolygonMode::Fill => vk::PolygonMode::FILL,
            PolygonMode::Line => vk::PolygonMode::LINE,
            PolygonMode::Point => vk::PolygonMode::POINT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FrontFace {
    CW,
    CCW,
}

impl Default for FrontFace {
    fn default() -> Self {
        Self::CCW
    }
}

impl Into<vk::FrontFace> for FrontFace {
    fn into(self) -> vk::FrontFace {
        match self {
            FrontFace::CW => vk::FrontFace::CLOCKWISE,
            FrontFace::CCW => vk::FrontFace::COUNTER_CLOCKWISE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CullMode {
    None,
    Front,
    Back,
    FrontBack,
}

impl Default for CullMode {
    fn default() -> Self {
        Self::Back
    }
}

impl Into<vk::CullModeFlags> for CullMode {
    fn into(self) -> vk::CullModeFlags {
        match self {
            CullMode::None => vk::CullModeFlags::NONE,
            CullMode::Front => vk::CullModeFlags::FRONT,
            CullMode::Back => vk::CullModeFlags::BACK,
            CullMode::FrontBack => vk::CullModeFlags::FRONT_AND_BACK,
        }
    }
}

struct RasterizationState {
    polygon_mode: PolygonMode,
    front_face: FrontFace,
    cull_mode: CullMode,
}

impl RasterizationState {
    fn new(polygon_mode: PolygonMode, front_face: FrontFace, cull_mode: CullMode) -> Self {
        RasterizationState {
            polygon_mode,
            front_face,
            cull_mode,
        }
    }

    fn info(&self) -> vk::PipelineRasterizationStateCreateInfoBuilder {
        vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.)
            .front_face(self.front_face.into())
            .cull_mode(self.cull_mode.into())
            .polygon_mode(self.polygon_mode.into())
    }
}

struct MultisampleState;

impl MultisampleState {
    fn new() -> Self {
        MultisampleState
    }

    fn info(&self) -> vk::PipelineMultisampleStateCreateInfoBuilder {
        vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
    }
}

#[allow(unused)]
struct StencilOpState;

#[allow(unused)]
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

#[derive(Clone, Copy, Debug)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    GreaterEqual,
    NotEqual,
    Always,
}

impl Into<vk::CompareOp> for CompareOp {
    fn into(self) -> vk::CompareOp {
        match self {
            CompareOp::Never => vk::CompareOp::NEVER,
            CompareOp::Less => vk::CompareOp::LESS,
            CompareOp::Equal => vk::CompareOp::EQUAL,
            CompareOp::LessEqual => vk::CompareOp::LESS_OR_EQUAL,
            CompareOp::Greater => vk::CompareOp::GREATER,
            CompareOp::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
            CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
            CompareOp::Always => vk::CompareOp::ALWAYS,
        }
    }
}

struct DepthStencilState {
    test_enable: bool,
    write_enable: bool,
    compare: CompareOp,
}

impl DepthStencilState {
    fn new(test_enable: bool, write_enable: bool, compare: CompareOp) -> Self {
        DepthStencilState {
            test_enable,
            write_enable,
            compare,
        }
    }

    fn info(&self) -> vk::PipelineDepthStencilStateCreateInfoBuilder {
        vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(self.test_enable)
            .depth_write_enable(self.write_enable)
            .depth_compare_op(self.compare.into())
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .min_depth_bounds(0.)
            .max_depth_bounds(1.)
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
            .color_write_mask(vk::ColorComponentFlags::RGBA)
    }
}

struct ColorBlendState {
    attachment_state: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl ColorBlendState {
    fn new(attachment_state: ColorBlendAttachmentState) -> Self {
        ColorBlendState {
            attachment_state: vec![attachment_state.info().build()],
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
    attribute_desc: Vec<vk::VertexInputAttributeDescription>,
}

impl VertexInputState {
    fn new(vertex_layout: &VertexLayout) -> Self {
        VertexInputState {
            binding_desc: vertex_layout.binding_description(),
            attribute_desc: vertex_layout.attribute_description(),
        }
    }

    fn info(&self) -> vk::PipelineVertexInputStateCreateInfoBuilder {
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&self.binding_desc)
            .vertex_attribute_descriptions(&self.attribute_desc)
    }
}

impl Default for VertexInputState {
    fn default() -> Self {
        Self {
            binding_desc: Default::default(),
            attribute_desc: Default::default(),
        }
    }
}

struct RenderingState {
    color_format: Vec<vk::Format>,
    depth_format: Option<vk::Format>,
}

impl RenderingState {
    fn new(color_format: Vec<ImageFormat>, depth_format: Option<ImageFormat>) -> Self {
        let color_format = color_format
            .iter()
            .map(|&f| f.into())
            .collect::<Vec<vk::Format>>();

        let depth_format = depth_format.map(|f| f.into());

        RenderingState {
            color_format,
            depth_format,
        }
    }

    fn info(&self) -> vk::PipelineRenderingCreateInfoBuilder {
        match self.depth_format {
            Some(depth_format) => vk::PipelineRenderingCreateInfo::builder()
                .color_attachment_formats(&self.color_format)
                .depth_attachment_format(depth_format.into()),
            None => vk::PipelineRenderingCreateInfo::builder()
                .color_attachment_formats(&self.color_format),
        }
    }
}

#[derive(Default)]
pub struct GraphicsPipelineBuilder {
    device: Option<Arc<Device>>,
    vertex_stage: Option<ShaderStage>,
    fragment_stage: Option<ShaderStage>,
    vertex_input_state: Option<VertexInputState>,
    viewports: Option<Vec<Viewport>>,
    scissors: Option<Vec<Rect2D>>,
    dynamic_states: Option<DynamicStates>,
    pipeline_layout: Option<Arc<PipelineLayout>>,
    rendering_state: Option<RenderingState>,
    depth_stencil: Option<DepthStencilState>,
    polygon_mode: PolygonMode,
    front_face: FrontFace,
    cull_mode: CullMode,
    primitive_topology: PrimitiveTopology,
}

impl GraphicsPipelineBuilder {
    pub fn new(device: Arc<Device>) -> Self {
        GraphicsPipelineBuilder {
            device: Some(device),
            ..Default::default()
        }
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

    pub fn layout(mut self, pipeline_layout: Arc<PipelineLayout>) -> Self {
        self.pipeline_layout = Some(pipeline_layout);

        self
    }

    pub fn polygon_mode(mut self, mode: PolygonMode) -> Self {
        self.polygon_mode = mode;

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

    pub fn primitive_topology(mut self, primitive_topology: PrimitiveTopology) -> Self {
        self.primitive_topology = primitive_topology;

        self
    }

    pub fn attachments(
        mut self,
        color_format: ImageFormat,
        depth_format: Option<ImageFormat>,
    ) -> Self {
        self.rendering_state = Some(RenderingState::new(
            vec![color_format.into()],
            depth_format.into(),
        ));

        self
    }

    pub fn depth_test_disable(mut self) -> Self {
        self.depth_stencil = Some(DepthStencilState::new(false, false, CompareOp::Never));

        self
    }

    pub fn build(mut self) -> Pipeline {
        let device = self.device.unwrap();

        let rendering_state = self.rendering_state.unwrap();
        let mut rendering_state_info = rendering_state.info();

        let vertex_stage = self.vertex_stage.unwrap();
        let vertex_stage_info = vertex_stage.info();

        let fragment_stage = self.fragment_stage.unwrap();
        let fragment_stage_info = fragment_stage.info();

        let shader_stages = [vertex_stage_info.build(), fragment_stage_info.build()];

        if self.vertex_input_state.is_none() {
            self.vertex_input_state = Some(VertexInputState::default());
        }

        let vertex_input_state = self.vertex_input_state.unwrap();
        let vertex_input_state_info = vertex_input_state.info();

        let viewports = self.viewports.unwrap();
        let scissors = self.scissors.unwrap();

        let viewport_state = ViewportState::new(&viewports, &scissors);
        let viewport_state_info = viewport_state.info();

        let dynamic_states = self.dynamic_states.unwrap();
        let dynamic_states_info = dynamic_states.info();

        let input_assembly_state = InputAssemblyState::new(self.primitive_topology);
        let input_assembly_state_info = input_assembly_state.info();

        let rasterization_state =
            RasterizationState::new(self.polygon_mode, self.front_face, self.cull_mode);
        let rasterization_state_info = rasterization_state.info();

        let multisample_state = MultisampleState::new();
        let multisample_state_info = multisample_state.info();

        let depth_stencil_state = self.depth_stencil.unwrap();
        let depth_stencil_state_info = depth_stencil_state.info();

        let color_blend_attachment = ColorBlendAttachmentState::new();
        let color_blend_state = ColorBlendState::new(color_blend_attachment);
        let color_blend_state_info = color_blend_state.info();

        let pipeline_layout = self.pipeline_layout.unwrap();

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .push_next(&mut rendering_state_info)
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_info)
            .viewport_state(&viewport_state_info)
            .dynamic_state(&dynamic_states_info)
            .input_assembly_state(&input_assembly_state_info)
            .rasterization_state(&rasterization_state_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_stencil_state_info)
            .color_blend_state(&color_blend_state_info)
            .layout(pipeline_layout.raw());

        let bind_point = vk::PipelineBindPoint::GRAPHICS;

        let pipeline = unsafe {
            device
                .raw()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_info.build()],
                    None,
                )
                .unwrap()[0]
        };

        Pipeline {
            device: device,
            layout: pipeline_layout,
            pipeline,
            bind_point,
        }
    }
}
