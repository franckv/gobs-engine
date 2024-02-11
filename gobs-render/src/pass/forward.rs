use std::sync::Arc;

use gobs_vulkan::{
    image::{Image, ImageExtent2D, ImageLayout},
    pipeline::Pipeline,
};

use crate::{context::Context, geometry::VertexFlag, graph::RenderError, CommandBuffer};

use super::{PassId, PassType, RenderPass};

pub struct ForwardPass {
    id: PassId,
    name: String,
    ty: PassType,
}

impl ForwardPass {
    pub fn new(name: &str) -> Arc<dyn RenderPass> {
        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Forward,
        })
    }
}

impl RenderPass for ForwardPass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> PassType {
        self.ty
    }

    fn pipeline(&self) -> Option<Arc<Pipeline>> {
        None
    }

    fn vertex_flags(&self) -> Option<VertexFlag> {
        None
    }

    fn render(
        self: Arc<Self>,
        _ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &dyn Fn(Arc<dyn RenderPass>, &CommandBuffer),
    ) -> Result<(), RenderError> {
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

        draw_cmd(self, cmd);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }
}
