use uuid::Uuid;

use gobs_core::memory::allocator::{Allocable, AllocableInfo, Allocator};
use gobs_render_hal::{BufferType, Handle, RenderHAL};

pub const STAGING_BUFFER_SIZE: usize = 1_048_576;

pub struct Buffer {
    pub id: Uuid,
    pub buffer: Handle,
    ty: BufferType,
    size: usize,
}

impl AllocableInfo<BufferType> for Buffer {
    fn resource_id(&self) -> uuid::Uuid {
        self.id
    }

    fn family(&self) -> BufferType {
        self.ty
    }

    fn resource_size(&self) -> usize {
        self.size
    }
}

impl<B: RenderHAL + ?Sized> Allocable<B, BufferType> for Buffer {
    fn allocate(hal: &mut B, name: &str, size: usize, family: BufferType) -> Self {
        let buffer = hal.create_buffer(name, size, family);

        Buffer {
            id: Uuid::new_v4(),
            buffer,
            ty: family,
            size,
        }
    }
}

pub struct BufferPool {
    buffer_pool: Allocator<BufferType, Buffer>,
}

#[allow(unused)]
impl BufferPool {
    pub fn new() -> Self {
        Self {
            buffer_pool: Allocator::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer_pool.is_empty()
    }

    pub fn allocate(
        &mut self,
        hal: &mut dyn RenderHAL,
        name: &str,
        size: usize,
        family: BufferType,
    ) -> &mut Buffer {
        let size = size.max(STAGING_BUFFER_SIZE);

        self.buffer_pool.allocate(hal, name, size, family).unwrap()
    }

    pub fn recycle(&mut self, id: &Uuid) {
        self.buffer_pool.recycle(id);
    }

    pub fn recycle_all(&mut self) {
        self.buffer_pool.recycle_all();
    }
}
