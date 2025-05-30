use gobs_core::Transform;
use gobs_render_graph::{RenderError, RenderPass};
use gobs_resource::manager::ResourceManager;

use crate::RenderBatch;

pub trait Renderable {
    fn draw(
        &self,
        resource_manager: &mut ResourceManager,
        pass: RenderPass,
        batch: &mut RenderBatch,
        transform: Option<Transform>,
    ) -> Result<(), RenderError>;
}
