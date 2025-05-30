use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::data::pool::ObjectPool;

pub trait ResourceFamily: Hash + Eq + Debug {}
impl<U: Hash + Eq + Debug> ResourceFamily for U {}

pub trait Allocable<D, F: ResourceFamily> {
    fn family(&self) -> F;
    fn size(&self) -> usize;
    fn allocate(device: &D, name: &str, size: usize, family: F) -> Self;
}

pub struct Allocator<D, F, A>
where
    F: ResourceFamily,
    A: Allocable<D, F>,
{
    pub pool: ObjectPool<F, A>,
    device: PhantomData<D>,
}

impl<D, F: ResourceFamily, A: Allocable<D, F>> Default for Allocator<D, F, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D, F: ResourceFamily, A: Allocable<D, F>> Allocator<D, F, A> {
    pub fn new() -> Self {
        Self {
            pool: ObjectPool::new(),
            device: PhantomData,
        }
    }

    #[tracing::instrument(target = "memory", skip_all, level = "trace")]
    pub fn allocate(&mut self, device: &D, name: &str, size: usize, family: F) -> A {
        while self.pool.contains(&family) {
            let resource = self.pool.pop(&family);

            if let Some(resource) = resource {
                if resource.size() >= size {
                    tracing::debug!(
                        "Reuse resource {:?}, {} ({})",
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
