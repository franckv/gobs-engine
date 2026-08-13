use gobs_core::Transform;
use gobs_render::{BoundingBox, RenderFlags, Renderable};

use crate::components::{BaseComponent, BoundingComponent, NodeId, NodeValue};

#[derive(Clone)]
pub struct Node {
    pub base: BaseComponent,
    pub bounding: BoundingComponent,
    pub(crate) transform: Transform,
    pub(crate) global_transform: Transform,
}

impl Default for Node {
    fn default() -> Self {
        let base = BaseComponent::default();
        let bounding = BoundingComponent::default();

        Self {
            base,
            bounding,
            transform: Transform::IDENTITY,
            global_transform: Transform::IDENTITY,
        }
    }
}

impl Node {
    pub(crate) fn new(
        value: NodeValue,
        transform: Transform,
        parent: Option<NodeId>,
        parent_transform: Transform,
    ) -> Self {
        let base = BaseComponent::new(value.clone(), parent);
        let bounding = BoundingComponent::new(value);

        Self {
            base,
            bounding,
            transform,
            global_transform: parent_transform * transform,
        }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn global_transform(&self) -> &Transform {
        &self.global_transform
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn update_transform<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Transform) -> bool,
    {
        self.base.updated |= f(&mut self.transform);
    }
}

impl Renderable for Node {
    fn draw(
        &self,
        ctx: &mut gobs_render::GfxContext,
        resource_manager: &mut gobs_resource::ResourceManager,
        batch: &mut gobs_render::RenderBatch,
        _transform: Option<gobs_core::Transform>,
        _bounding_box: Option<BoundingBox>,
        render_flags: gobs_render::RenderFlags,
    ) -> Result<(), gobs_resource::ResourceError> {
        match &self.base.value  {
            NodeValue::None => (),
            NodeValue::Camera(_camera) => (),
            NodeValue::Light(_light) => (),
            NodeValue::Model(model) => {
                let mut render_flags = render_flags;

                if self.base.selected {
                    render_flags |= RenderFlags::SELECTED;
                }

                model.draw(
                    ctx,
                    resource_manager,
                    batch,
                    Some(*self.global_transform()),
                    Some(self.bounding.bounding_box),
                    render_flags,
                )?;
            }
        }

        Ok(())
    }
}
