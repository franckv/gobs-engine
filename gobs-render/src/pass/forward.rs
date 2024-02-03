use gobs_vulkan::image::{Image, ImageExtent2D, ImageLayout};

use crate::CommandBuffer;

use super::{PassType, RenderPass};

pub struct ForwardPass {
    name: String,
    ty: PassType,
}

impl ForwardPass {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: PassType::Forward,
        }
    }
}

impl RenderPass for ForwardPass {
    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn render<F>(
        &self,
        _ctx: &crate::context::Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &F,
    ) -> Result<(), crate::graph::RenderError>
    where
        F: Fn(PassType, &str, &crate::CommandBuffer),
    {
        log::debug!("Draw forward");

        cmd.begin_label("Draw forward");

        cmd.transition_image_layout(&mut render_targets[0], ImageLayout::Color);
        cmd.transition_image_layout(&mut render_targets[1], ImageLayout::Depth);

        cmd.begin_rendering(
            &render_targets[0],
            draw_extent,
            Some(&render_targets[1]),
            false,
            [0.; 4],
            1.,
        );

        cmd.set_viewport(draw_extent.width, draw_extent.height);

        draw_cmd(self.ty, &self.name, cmd);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }
}
