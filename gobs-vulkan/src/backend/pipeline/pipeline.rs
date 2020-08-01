use std::ptr;
use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::backend::descriptor::DescriptorSetLayout;
use crate::backend::device::Device;
use crate::backend::pipeline::{Shader, VertexLayout};
use crate::backend::renderpass::RenderPass;
use crate::backend::Wrap;

const WIDTH: u32 = 1400;
const HEIGHT: u32 = 1050;

/*
pub struct PipelineBuilder {
    device: Arc<Device>,
    renderpass: Arc<RenderPass>
}

impl PipelineBuilder {
    pub fn new(device: Arc<Device>,
               renderpass: Arc<RenderPass>) -> Self {
        PipelineBuilder {
            device,
            renderpass
        }
    }

    pub fn build(self) -> Pipeline {
        Pipeline {
            device: self.device,
            _renderpass: self.renderpass
        }
    }
}
*/

pub struct Pipeline {
    device: Arc<Device>,
    _renderpass: Arc<RenderPass>,
    pub(crate) layout: vk::PipelineLayout,
    _descriptor_layout: Arc<DescriptorSetLayout>,
    pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn new(device: Arc<Device>,
               vshader: Shader, fshader: Shader,
               vertex_layout: VertexLayout,
               descriptor_layout: Arc<DescriptorSetLayout>,
               renderpass: Arc<RenderPass>,
               subpass: u32) -> Self {
        let entry = CString::new("main").unwrap();

        let vertex_stage_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vshader.raw(),
            p_name: entry.as_ptr(),
            p_specialization_info: ptr::null(),
        };

        let fragment_stage_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: fshader.raw(),
            p_name: entry.as_ptr(),
            p_specialization_info: ptr::null(),
        };

        let shader_stages = [vertex_stage_info, fragment_stage_info];

        let binding_desc =
            Self::get_binding_description(&vertex_layout);

        let attribute_desc =
            Self::get_attribute_description(&vertex_layout);

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            vertex_binding_description_count: binding_desc.len() as u32,
            p_vertex_binding_descriptions: binding_desc.as_ptr(),
            vertex_attribute_description_count: attribute_desc.len() as u32,
            p_vertex_attribute_descriptions: attribute_desc.as_ptr(),
        };

        let assembly_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: 0,
        };

        let viewports = [
            vk::Viewport {
                x: 0.,
                y: 0.,
                width: WIDTH as f32,
                height: HEIGHT as f32,
                min_depth: 0.,
                max_depth: 1.,
            }
        ];

        let scissors = [
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width: WIDTH, height: HEIGHT },
            }
        ];

        let viewport_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
        };

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
        };

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            depth_clamp_enable: 0,
            rasterizer_discard_enable: 0,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: 0,
            depth_bias_constant_factor: 0.,
            depth_bias_clamp: 0.,
            depth_bias_slope_factor: 0.,
        };

        let multisample_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            sample_shading_enable: 0,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: 0,
            alpha_to_one_enable: 0,
        };

        let color_blend_attachment = [
            vk::PipelineColorBlendAttachmentState {
                color_write_mask: vk::ColorComponentFlags::all(),
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::ONE,
                dst_color_blend_factor: vk::BlendFactor::ZERO,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
            }
        ];

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            logic_op_enable: 0,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: color_blend_attachment.as_ptr(),
            blend_constants: [0., 0., 0., 0.],
        };

        let layout = Self::get_layout(&device, &descriptor_layout);

        let op_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: 0,
            min_depth_bounds: 0.,
            max_depth_bounds: 1.,
            stencil_test_enable: 0,
            front: op_state.clone(),
            back: op_state
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &assembly_info,
            p_viewport_state: &viewport_info,
            p_rasterization_state: &rasterization_info,
            p_multisample_state: &multisample_info,
            p_depth_stencil_state: &depth_info,
            p_color_blend_state: &color_blend_info,
            p_dynamic_state: &dynamic_info,
            p_tessellation_state: ptr::null(),
            layout,
            render_pass: renderpass.raw(),
            subpass,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
        };

        let pipeline = unsafe {
            device.raw().create_graphics_pipelines(vk::PipelineCache::null(),
                                                   &[pipeline_info], None).unwrap()[0]
        };
        Pipeline {
            device,
            _renderpass: renderpass,
            layout,
            _descriptor_layout: descriptor_layout,
            pipeline,
        }
    }

    fn get_layout(device: &Arc<Device>, descriptor_layout: &DescriptorSetLayout)
                  -> vk::PipelineLayout {
        let set_layout = [descriptor_layout.layout];

        let layout_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            set_layout_count: 1,
            p_set_layouts: set_layout.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        unsafe {
            device.raw().create_pipeline_layout(&layout_info,
                                                None).unwrap()
        }
    }

    fn get_binding_description(vertex_layout: &VertexLayout)
                               -> Vec<vk::VertexInputBindingDescription> {
        let mut desc = Vec::new();

        for binding in &vertex_layout.bindings {
            desc.push(
                vk::VertexInputBindingDescription {
                    binding: binding.binding as u32,
                    stride: binding.stride as u32,
                    input_rate: binding.ty.into()
                });
        }

        desc
    }

    fn get_attribute_description(vertex_layout: &VertexLayout)
                                 -> Vec<vk::VertexInputAttributeDescription> {
        let mut desc = Vec::new();

        for binding in &vertex_layout.bindings {
            for attr in &binding.attributes {
                for i in 0..attr.format.locations() {
                    desc.push(vk::VertexInputAttributeDescription {
                        binding: binding.binding as u32,
                        location: (attr.location + i) as u32,
                        format: attr.format.into(),
                        offset: (attr.offset +
                            i * attr.format.location_size()) as u32
                    });
                }
            }
        }

        desc
    }
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
            self.device.raw().destroy_pipeline_layout(self.layout, None);
        }
    }
}
