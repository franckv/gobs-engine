use std::collections::{HashMap, hash_map::Entry};

use bitflags::bitflags;

use gobs_render_hal::ImageLayout;

use crate::{
    PassId,
    pass::{AttachmentAccess, AttachmentType},
};

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
pub struct SyncScope {
    pub stage: BarrierStage,
    pub access: BarrierAccess,
}

impl SyncScope {
    pub fn stage_only(&self) -> SyncScope {
        SyncScope {
            stage: self.stage,
            access: BarrierAccess::empty(),
        }
    }
}

pub struct SyncStatus {
    last_write: SyncScope,
    last_layout: ImageLayout,
    invalidates: HashMap<BarrierStage, BarrierAccess>,
}

impl SyncStatus {
    pub fn new(scope: SyncScope, layout: ImageLayout) -> Self {
        Self {
            last_write: scope,
            last_layout: layout,
            invalidates: HashMap::new(),
        }
    }

    pub fn last_write(&self) -> SyncScope {
        self.last_write
    }

    pub fn last_layout(&self) -> ImageLayout {
        self.last_layout
    }

    pub fn update(&mut self, scope: SyncScope, layout: ImageLayout) {
        self.last_write = scope;
        self.last_layout = layout;
    }

    pub fn invalidate(&mut self, scope: SyncScope) {
        let access = scope.access.intersection(ALL_READS);

        if access.is_empty() {
            return;
        }

        for stage in scope.stage {
            match self.invalidates.entry(stage) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() |= access;
                }
                Entry::Vacant(e) => {
                    e.insert(access);
                }
            }
        }
    }

    pub fn is_invalidated(&self, scope: SyncScope) -> bool {
        let access = scope.access.intersection(ALL_READS);

        if access.is_empty() {
            return false;
        }

        let mut result = false;
        for stage in scope.stage {
            match self.invalidates.get(&stage) {
                Some(a) => {
                    if a.contains(access) {
                        result = true;
                    } else {
                        result = false;
                        break;
                    }
                }
                None => {
                    result = false;
                    break;
                }
            }
        }

        result
    }

    pub fn is_flushed(&self) -> bool {
        !self.invalidates.is_empty()
    }

    pub fn clear_invalidates(&mut self) {
        self.invalidates.clear();
    }
}

#[derive(Clone, Debug)]
pub struct Barrier {
    pub attachment: String,
    pub pass_id: PassId,
    pub src_layout: ImageLayout,
    pub dst_layout: ImageLayout,
    pub src_scope: SyncScope,
    pub dst_scope: SyncScope,
}

impl Barrier {
    pub fn new(attachment: &str, pass_id: PassId) -> Self {
        let empty = SyncScope {
            stage: BarrierStage::empty(),
            access: BarrierAccess::empty(),
        };

        Self {
            attachment: attachment.to_string(),
            pass_id,
            src_layout: ImageLayout::Undefined,
            dst_layout: ImageLayout::Undefined,
            src_scope: empty,
            dst_scope: empty,
        }
    }

    pub fn layouts(mut self, src_layout: ImageLayout, dst_layout: ImageLayout) -> Self {
        self.src_layout = src_layout;
        self.dst_layout = dst_layout;

        self
    }

    pub fn memory(mut self, src_scope: SyncScope, dst_scope: SyncScope) -> Self {
        self.src_scope = src_scope; // no flush
        self.dst_scope = dst_scope;

        self
    }

    pub fn invalidation(mut self, src_scope: SyncScope, dst_scope: SyncScope) -> Self {
        self.src_scope = src_scope.stage_only(); // no flush
        self.dst_scope = dst_scope;

        self
    }

    pub fn flush(mut self, src_scope: SyncScope, dst_scope: SyncScope) -> Self {
        self.src_scope = src_scope;
        self.dst_scope = dst_scope.stage_only(); // no invalidate

        self
    }

    pub fn execution(mut self, src_scope: SyncScope, dst_scope: SyncScope) -> Self {
        self.src_scope = src_scope.stage_only(); // no flush
        self.dst_scope = dst_scope.stage_only(); // no invalidate

        self
    }

    pub fn barrier_scope(ty: AttachmentType, access: AttachmentAccess) -> SyncScope {
        match (ty, access) {
            (AttachmentType::Color, AttachmentAccess::Read) => SyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentRead,
            },
            (AttachmentType::Color, AttachmentAccess::Write) => SyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentWrite,
            },
            (AttachmentType::Color, AttachmentAccess::ReadWrite) => SyncScope {
                stage: BarrierStage::ColorAttachmentOutput,
                access: BarrierAccess::ColorAttachmentRead | BarrierAccess::ColorAttachmentWrite,
            },
            (AttachmentType::Depth, AttachmentAccess::Read) => SyncScope {
                stage: BarrierStage::FragmentTests,
                access: BarrierAccess::DepthStencilAttachmentRead,
            },
            (AttachmentType::Depth, AttachmentAccess::Write) => SyncScope {
                stage: BarrierStage::FragmentTests,
                access: BarrierAccess::DepthStencilAttachmentWrite,
            },
            (AttachmentType::Depth, AttachmentAccess::ReadWrite) => SyncScope {
                stage: BarrierStage::FragmentTests,
                access: BarrierAccess::DepthStencilAttachmentRead
                    | BarrierAccess::DepthStencilAttachmentWrite,
            },
            (AttachmentType::ImageStorage, AttachmentAccess::Read) => SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead,
            },
            (AttachmentType::ImageStorage, AttachmentAccess::Write) => SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageWrite,
            },
            (AttachmentType::ImageStorage, AttachmentAccess::ReadWrite) => SyncScope {
                stage: BarrierStage::ComputeShader,
                access: BarrierAccess::ShaderStorageRead | BarrierAccess::ShaderStorageWrite,
            },
        }
    }
}
