use gobs_core::Transform;
use gobs_render_graph::{GfxContext, RenderPass};
use gobs_resource::{ResourceError, ResourceManager};

use crate::RenderBatch;

pub trait Renderable {
    fn draw(
        &self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
    ) -> Result<(), ResourceError>;
}
