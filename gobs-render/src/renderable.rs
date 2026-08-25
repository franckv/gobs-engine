use gobs_core::Transform;
use gobs_render_graph::GfxContext;
use gobs_resource::{ResourceError, ResourceManager};

use crate::{BoundingBox, RenderBatch, data::RenderFlags};

pub trait Renderable {
    fn draw(
        &self,
        ctx: &mut GfxContext,
        resource_manager: &mut ResourceManager,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
        bounding_box: Option<BoundingBox>,
        render_flags: RenderFlags,
    ) -> Result<(), ResourceError>;
}
