use ash::vk;
use bitflags::bitflags;

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

impl From<BarrierAccess> for vk::AccessFlags2 {
    fn from(value: BarrierAccess) -> Self {
        let mut access = vk::AccessFlags2::empty();

        value.iter().for_each(|a| match a {
            BarrierAccess::ShaderRead => access |= vk::AccessFlags2::SHADER_READ,
            BarrierAccess::ShaderWrite => access |= vk::AccessFlags2::SHADER_WRITE,
            BarrierAccess::ColorAttachmentRead => access |= vk::AccessFlags2::COLOR_ATTACHMENT_READ,
            BarrierAccess::ColorAttachmentWrite => {
                access |= vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
            }
            BarrierAccess::DepthStencilAttachmentRead => {
                access |= vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
            }
            BarrierAccess::DepthStencilAttachmentWrite => {
                access |= vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
            }
            BarrierAccess::ShaderSampledRead => access |= vk::AccessFlags2::SHADER_SAMPLED_READ,
            BarrierAccess::ShaderStorageRead => access |= vk::AccessFlags2::SHADER_STORAGE_READ,
            BarrierAccess::ShaderStorageWrite => access |= vk::AccessFlags2::SHADER_STORAGE_WRITE,
            _ => unimplemented!(),
        });

        access
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

impl From<BarrierStage> for vk::PipelineStageFlags2 {
    fn from(value: BarrierStage) -> Self {
        let mut stage = vk::PipelineStageFlags2::empty();

        value.iter().for_each(|s| match s {
            BarrierStage::TopOfPipe => stage |= vk::PipelineStageFlags2::TOP_OF_PIPE,
            BarrierStage::ComputeShader => stage |= vk::PipelineStageFlags2::COMPUTE_SHADER,
            BarrierStage::FragmentShader => stage |= vk::PipelineStageFlags2::FRAGMENT_SHADER,
            BarrierStage::FragmentTests => {
                stage |= vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS
            }
            BarrierStage::ColorAttachmentOutput => {
                stage |= vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT
            }
            BarrierStage::BottomOfPipe => stage |= vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
            _ => unimplemented!(),
        });

        stage
    }
}
