use bitflags::bitflags;
use gobs_vulkan::images::ImageLayout;

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
    pub struct BarrierAccess: u32 {
        const ShaderRead = 1;
        const ShaderWrite = 1 << 1;
        const ColorAttachmentRead = 1 << 2;
        const ColorAttachmentWrite = 1 << 3;
        const DepthStencilAttachmentRead = 1 << 4;
        const DepthStencilAttachmentWrite = 1 << 5;
        const ShaderSampledRead = 1 << 6;
        const ShaderStorageRead = 1 << 7;
        const ShaderStorageWrite = 1 << 8;
    }
}

const ALL_READS: BarrierAccess = BarrierAccess::ShaderRead
    .union(BarrierAccess::ColorAttachmentRead)
    .union(BarrierAccess::DepthStencilAttachmentRead)
    .union(BarrierAccess::ShaderStorageRead)
    .union(BarrierAccess::ShaderSampledRead);

const ALL_WRITES: BarrierAccess = BarrierAccess::ShaderWrite
    .union(BarrierAccess::ColorAttachmentWrite)
    .union(BarrierAccess::DepthStencilAttachmentWrite)
    .union(BarrierAccess::ShaderStorageWrite);

impl BarrierAccess {
    pub fn is_write(&self) -> bool {
        self.intersects(ALL_WRITES)
    }

    pub fn reads(&self) -> BarrierAccess {
        self.intersection(ALL_READS)
    }

    pub fn writes(&self) -> BarrierAccess {
        self.intersection(ALL_WRITES)
    }
}

bitflags! {
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
    pub struct BarrierStage: u32 {
        const TopOfPipe = 1;
        const ComputeShader = 1 << 1;
        const FragmentShader = 1 << 2;
        const FragmentTests = 1 << 3;
        const ColorAttachmentOutput = 1 << 4;
        const BottomOfPipe = 1 << 5;
        }
}

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
