use uuid::Uuid;

use gobs_core::memory::allocator::Allocable;
use gobs_render_hal::{BufferType, Handle, RenderHAL};

pub const STAGING_BUFFER_SIZE: usize = 1_048_576;

pub struct Buffer {
    pub id: Uuid,
    pub buffer: Handle,
    ty: BufferType,
    size: usize,
}

impl Allocable<Box<dyn RenderHAL>, BufferType> for Buffer {
    fn resource_id(&self) -> uuid::Uuid {
        self.id
    }

    fn family(&self) -> BufferType {
        self.ty
    }

    fn resource_size(&self) -> usize {
        self.size
    }

    fn allocate(hal: &mut Box<dyn RenderHAL>, name: &str, size: usize, family: BufferType) -> Self {
        let buffer = hal.create_buffer(name, size, family);

        Buffer {
            id: Uuid::new_v4(),
            buffer,
            ty: family,
            size,
        }
    }
}
