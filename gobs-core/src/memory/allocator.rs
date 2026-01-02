use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use thiserror::Error;
use uuid::Uuid;

use crate::data::pool::ObjectPool;
use crate::logger;
use crate::memory::buddy::BuddyAllocator;

#[derive(Error, Debug)]
pub enum AllocationError {
    #[error("allocation failure")]
    AllocationFailure,
}

pub trait ResourceFamily: Hash + Eq + Debug {}
impl<U: Hash + Eq + Debug> ResourceFamily for U {}

pub trait Allocable<B, F: ResourceFamily> {
    fn resource_id(&self) -> Uuid;
    fn family(&self) -> F;
    fn resource_size(&self) -> usize;
    fn allocate(backend: &B, name: &str, size: usize, family: F) -> Self;
}

pub struct AllocableBlock<B, F, A>
where
    F: ResourceFamily,
    A: Allocable<B, F>,
{
    allocable: A,
    block_allocator: BuddyAllocator,
    backend: PhantomData<B>,
    family: PhantomData<F>,
}

impl<B, F: ResourceFamily, A: Allocable<B, F>> AllocableBlock<B, F, A> {
    pub fn new(backend: &B, name: &str, size: usize, family: F) -> Self {
        Self {
            allocable: A::allocate(backend, name, size, family),
            block_allocator: BuddyAllocator::new(size, 5).unwrap(),
            backend: std::marker::PhantomData,
            family: std::marker::PhantomData,
        }
    }

    pub fn max_size(&self) -> usize {
        self.block_allocator.max_available_size()
    }
}

pub struct Allocator<B, F, A>
where
    F: ResourceFamily,
    A: Allocable<B, F>,
{
    pub pool: ObjectPool<F, AllocableBlock<B, F, A>>,
    pub allocated: HashMap<Uuid, AllocableBlock<B, F, A>>,
    backend: PhantomData<B>,
}

impl<B, F: ResourceFamily, A: Allocable<B, F>> Default for Allocator<B, F, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B, F: ResourceFamily, A: Allocable<B, F>> Allocator<B, F, A> {
    pub fn new() -> Self {
        Self {
            pool: ObjectPool::new(),
            allocated: HashMap::new(),
            backend: PhantomData,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn allocate(
        &mut self,
        backend: &B,
        name: &str,
        size: usize,
        family: F,
    ) -> Result<&mut A, AllocationError> {
        while self.pool.contains(&family) {
            let resource = self.pool.pop(&family);

            if let Some(resource) = resource
                && resource.max_size() >= size
            {
                tracing::debug!(
                    target: logger::RENDER,
                    "Reuse resource {:?}, {} ({})",
                    family,
                    size,
                    self.pool.get(&family).unwrap().len()
                );

                let id = resource.allocable.resource_id();
                self.allocated.insert(id, resource);

                return self
                    .allocated
                    .get_mut(&id)
                    .map(|resource| &mut resource.allocable)
                    .ok_or(AllocationError::AllocationFailure);
            }
        }

        tracing::debug!(target: logger::MEMORY, "Allocate new resource {:?}, {}", family, size);
        let resource: AllocableBlock<B, F, A> = AllocableBlock::new(backend, name, size, family);
        let id = resource.allocable.resource_id();

        self.allocated
            .insert(resource.allocable.resource_id(), resource);

        self.allocated
            .get_mut(&id)
            .map(|resource| &mut resource.allocable)
            .ok_or(AllocationError::AllocationFailure)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn recycle(&mut self, id: &Uuid) {
        let resource = self.allocated.remove(id);
        if let Some(resource) = resource {
            self.pool.insert(resource.allocable.family(), resource);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.allocated.is_empty()
    }
}
