use std::sync::Arc;

use ash::vk;

use gobs_core::{ImageFormat, logger};

use crate::Wrap;
use crate::device::Device;
use crate::images::VkFormat;

pub struct RenderPass {
    device: Arc<Device>,
    renderpass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: Arc<Device>, format: ImageFormat, depth_format: ImageFormat) -> Arc<Self> {
        let color_attach = vk::AttachmentDescription::default()
            .format(VkFormat::from(format).into())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let depth_attach = vk::AttachmentDescription::default()
            .format(VkFormat::from(depth_format).into())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_ref))
            .depth_stencil_attachment(&depth_ref);

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            );

        let attachments = [color_attach, depth_attach];
        let renderpass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let renderpass = unsafe {
            tracing::debug!(target: logger::RENDER, "Create renderpass");
            device
                .raw()
                .create_render_pass(&renderpass_info, None)
                .unwrap()
        };

        Arc::new(RenderPass { device, renderpass })
    }
}

impl Wrap<vk::RenderPass> for RenderPass {
    fn raw(&self) -> vk::RenderPass {
        self.renderpass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        tracing::trace!(target: logger::MEMORY, "Drop renderpass");
        unsafe {
            self.device.raw().destroy_render_pass(self.renderpass, None);
        }
    }
}
