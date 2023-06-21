use std::ptr;
use std::sync::Arc;

use ash::vk;

use log::trace;

use crate::device::Device;
use crate::image::Image;
use crate::renderpass::RenderPass;
use crate::Wrap;

pub struct Framebuffer {
    device: Arc<Device>,
    framebuffer: vk::Framebuffer,
    image: Image,
    renderpass: Arc<RenderPass>
}

impl Framebuffer {
    pub fn new(device: Arc<Device>,
               image: Image,
               depth_buffer: &Image,
               renderpass: Arc<RenderPass>) -> Self {
        let attachments = [image.image_view, depth_buffer.image_view];

        let framebuffer_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            render_pass: renderpass.raw(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: image.width,
            height: image.height,
            layers: 1,
        };

        let framebuffer = unsafe {
            device.raw().create_framebuffer(&framebuffer_info,
                                             None).unwrap()
        };

        Framebuffer {
            device,
            framebuffer,
            image,
            renderpass
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    pub fn renderpass(&self) -> &Arc<RenderPass> {
        &self.renderpass
    }
}

impl Wrap<vk::Framebuffer> for Framebuffer {
    fn raw(&self) -> vk::Framebuffer {
        self.framebuffer
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        trace!("Drop framebuffer");
        unsafe {
            self.device.raw().destroy_framebuffer(self.framebuffer, None);
        }
    }
}
