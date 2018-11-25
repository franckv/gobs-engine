use std::ptr;
use std::sync::Arc;

use ash::vk;
use ash::version::DeviceV1_0;

use backend::device::Device;
use backend::image::ImageFormat;
use backend::Wrap;

pub struct RenderPass {
    device: Arc<Device>,
    renderpass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: Arc<Device>, format: ImageFormat) -> Arc<Self> {
        let color_attach = vk::AttachmentDescription {
            format: format.into(),
            flags: Default::default(),
            samples: vk::SAMPLE_COUNT_1_BIT,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::PresentSrcKhr,
        };

        let depth_attach = vk::AttachmentDescription {
            format: vk::Format::D32Sfloat,
            flags: Default::default(),
            samples: vk::SAMPLE_COUNT_1_BIT,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::DontCare,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::DepthStencilAttachmentOptimal,
        };

        let color_ref = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::ColorAttachmentOptimal,
            }
        ];

        let depth_ref = [
            vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::DepthStencilAttachmentOptimal
            }
        ];

        let subpass = vk::SubpassDescription {
            flags: Default::default(),
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            color_attachment_count: 1,
            p_color_attachments: color_ref.as_ptr(),
            p_depth_stencil_attachment: depth_ref.as_ptr(),
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
        };

        let dependency = vk::SubpassDependency {
            dependency_flags: Default::default(),
            src_subpass: vk::VK_SUBPASS_EXTERNAL,
            dst_subpass: Default::default(),
            src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            src_access_mask: Default::default(),
            dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT |
                vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
        };

        let attachments = [color_attach, depth_attach];

        let renderpass_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RenderPassCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            attachment_count: 2,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
        };

        let renderpass = unsafe {
            device.raw().create_render_pass(&renderpass_info, None).unwrap()
        };

        Arc::new(RenderPass {
            device,
            renderpass
        })
    }
}

impl Wrap<vk::RenderPass> for RenderPass {
    fn raw(&self) -> vk::RenderPass {
        self.renderpass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        trace!("Drop renderpass");
        unsafe {
            self.device.raw().destroy_render_pass(self.renderpass, None);
        }
    }
}
