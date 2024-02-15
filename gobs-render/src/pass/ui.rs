use std::sync::Arc;

use gobs_core::entity::uniform::{UniformLayout, UniformProp};
use gobs_vulkan::{
    image::{Image, ImageExtent2D, ImageLayout},
    pipeline::Pipeline,
};

use crate::{
    context::Context,
    geometry::VertexFlag,
    graph::RenderError,
    pass::{PassId, PassType, RenderPass},
    CommandBuffer,
};

pub struct UiPass {
    id: PassId,
    name: String,
    ty: PassType,
    push_layout: Arc<UniformLayout>,
}

impl UiPass {
    pub fn new(name: &str) -> Arc<dyn RenderPass> {
        let push_layout = UniformLayout::builder()
            .prop("vertex_buffer_address", UniformProp::U64)
            .build();

        Arc::new(Self {
            id: PassId::new_v4(),
            name: name.to_string(),
            ty: PassType::Ui,
            push_layout,
        })
    }
}

impl RenderPass for UiPass {
    fn id(&self) -> PassId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ty(&self) -> super::PassType {
        self.ty
    }

    fn pipeline(&self) -> Option<Arc<Pipeline>> {
        None
    }

    fn vertex_flags(&self) -> Option<VertexFlag> {
        None
    }

    fn push_layout(&self) -> Option<Arc<UniformLayout>> {
        Some(self.push_layout.clone())
    }

    fn render(
        self: Arc<Self>,
        _ctx: &Context,
        cmd: &CommandBuffer,
        render_targets: &mut [&mut Image],
        draw_extent: ImageExtent2D,
        draw_cmd: &dyn Fn(Arc<dyn RenderPass>, &CommandBuffer),
    ) -> Result<(), RenderError> {
        log::debug!("Draw UI");

        cmd.begin_label("Draw UI");

        cmd.transition_image_layout(&mut render_targets[0], ImageLayout::Color);

        cmd.begin_rendering(&render_targets[0], draw_extent, None, false, [0.; 4], 1.);

        cmd.set_viewport(draw_extent.width, draw_extent.height);

        draw_cmd(self, cmd);

        cmd.end_rendering();

        cmd.end_label();

        Ok(())
    }
}
