use std::fmt::Debug;
use std::hash::Hash;

use gobs_core::utils::pool::ObjectPool;
use gobs_gfx::{Buffer, BufferUsage, GfxBuffer, GfxDevice};

pub trait ResourceFamily: Hash + Eq + Debug {}
impl<U: Hash + Eq + Debug> ResourceFamily for U {}

pub trait Allocable<F: ResourceFamily> {
    fn family(&self) -> F;
    fn size(&self) -> usize;
    fn allocate(device: &GfxDevice, name: &str, size: usize, family: F) -> Self;
}

impl Allocable<BufferUsage> for GfxBuffer {
    fn allocate(device: &GfxDevice, name: &str, size: usize, family: BufferUsage) -> Self {
        GfxBuffer::new(name, size, family, device)
    }

    fn family(&self) -> BufferUsage {
        self.usage()
    }

    fn size(&self) -> usize {
        Buffer::size(self)
    }
}

pub struct Allocator<F: ResourceFamily, A: Allocable<F>> {
    pub pool: ObjectPool<F, A>,
}

impl<F: ResourceFamily, A: Allocable<F>> Allocator<F, A> {
    pub fn new() -> Self {
        Self {
            pool: ObjectPool::new(),
        }
    }

    #[tracing::instrument(target = "memory", skip_all, level = "trace")]
    pub fn allocate(&mut self, device: &GfxDevice, name: &str, size: usize, family: F) -> A {
        while self.pool.contains(&family) {
            let resource = self.pool.pop(&family);

            if let Some(resource) = resource {
                if resource.size() >= size {
                    tracing::debug!(
                        "Reuse buffer {:?}, {} ({})",
                        family,
                        size,
                        self.pool.get(&family).unwrap().len()
                    );

                    return resource;
                }
            }
        }

        tracing::debug!(target: "memory", "Allocate new resource {:?}, {}", family, size);
        A::allocate(device, name, size, family)
    }

    #[tracing::instrument(target = "memory", skip_all, level = "trace")]
    pub fn recycle(&mut self, resource: A) {
        self.pool.insert(resource.family(), resource);
    }
}
