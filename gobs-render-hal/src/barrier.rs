use gobs_vulkan::{
    images::ImageLayout,
    sync::{BarrierAccess, BarrierStage},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BarrierSyncScope {
    pub stage: BarrierStage,
    pub access: BarrierAccess,
}

impl BarrierSyncScope {
    pub fn stage_only(&self) -> BarrierSyncScope {
        BarrierSyncScope {
            stage: self.stage,
            access: BarrierAccess::empty(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BarrierType {
    Global,
    Image(String),
    Buffer(String),
}

#[derive(Clone, Debug)]
pub struct Barrier {
    pub label: String,
    pub ty: BarrierType,
    pub src_layout: ImageLayout,
    pub dst_layout: ImageLayout,
    pub src_scope: BarrierSyncScope,
    pub dst_scope: BarrierSyncScope,
}

impl Barrier {
    pub fn new(label: &str) -> Self {
        let empty = BarrierSyncScope {
            stage: BarrierStage::empty(),
            access: BarrierAccess::empty(),
        };

        Self {
            label: label.to_string(),
            ty: BarrierType::Global,
            src_layout: ImageLayout::Undefined,
            dst_layout: ImageLayout::Undefined,
            src_scope: empty,
            dst_scope: empty,
        }
    }

    pub fn image(mut self, image: &str) -> Self {
        self.ty = BarrierType::Image(image.to_string());

        self
    }

    pub fn buffer(mut self, buffer: &str) -> Self {
        self.ty = BarrierType::Buffer(buffer.to_string());

        self
    }

    pub fn layouts(mut self, src_layout: ImageLayout, dst_layout: ImageLayout) -> Self {
        self.src_layout = src_layout;
        self.dst_layout = dst_layout;

        self
    }

    pub fn memory(mut self, src_scope: BarrierSyncScope, dst_scope: BarrierSyncScope) -> Self {
        self.src_scope = src_scope;
        self.dst_scope = dst_scope;

        self
    }

    pub fn invalidation(
        mut self,
        src_scope: BarrierSyncScope,
        dst_scope: BarrierSyncScope,
    ) -> Self {
        self.src_scope = src_scope.stage_only(); // no flush
        self.dst_scope = dst_scope;

        self
    }

    pub fn flush(mut self, src_scope: BarrierSyncScope, dst_scope: BarrierSyncScope) -> Self {
        self.src_scope = src_scope;
        self.dst_scope = dst_scope.stage_only(); // no invalidate

        self
    }

    pub fn execution(mut self, src_scope: BarrierSyncScope, dst_scope: BarrierSyncScope) -> Self {
        self.src_scope = src_scope.stage_only(); // no flush
        self.dst_scope = dst_scope.stage_only(); // no invalidate

        self
    }
}
