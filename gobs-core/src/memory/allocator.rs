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

pub trait AllocableInfo<F: ResourceFamily> {
    fn resource_id(&self) -> Uuid;
    fn family(&self) -> F;
    fn resource_size(&self) -> usize;
}

pub trait Allocable<B: ?Sized, F: ResourceFamily>: AllocableInfo<F> {
    fn allocate(backend: &mut B, name: &str, size: usize, family: F) -> Self;
}

pub struct AllocableBlock<F, A>
where
    F: ResourceFamily,
{
    allocable: A,
    block_allocator: BuddyAllocator,
    family: PhantomData<F>,
}

impl<F: ResourceFamily, A> AllocableBlock<F, A> {
    pub fn new<B: ?Sized>(backend: &mut B, name: &str, size: usize, family: F) -> Self
    where
        A: Allocable<B, F>,
    {
        Self {
            allocable: A::allocate(backend, name, size, family),
            block_allocator: BuddyAllocator::new(size, 5).unwrap(),
            family: std::marker::PhantomData,
        }
    }

    pub fn max_size(&self) -> usize {
        self.block_allocator.max_available_size()
    }
}

pub struct Allocator<F, A>
where
    F: ResourceFamily,
{
    pub pool: ObjectPool<F, AllocableBlock<F, A>>,
    pub allocated: HashMap<Uuid, AllocableBlock<F, A>>,
}

impl<F: ResourceFamily, A: AllocableInfo<F>> Default for Allocator<F, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: ResourceFamily, A: AllocableInfo<F>> Allocator<F, A> {
    pub fn new() -> Self {
        Self {
            pool: ObjectPool::new(),
            allocated: HashMap::new(),
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn allocate<B: ?Sized>(
        &mut self,
        backend: &mut B,
        name: &str,
        size: usize,
        family: F,
    ) -> Result<&mut A, AllocationError>
    where
        A: Allocable<B, F>,
    {
        while self.pool.contains(&family) {
            let resource = self.pool.pop(&family);

            if let Some(resource) = resource
                && resource.max_size() >= size
            {
                tracing::debug!(
                    target: logger::MEMORY,
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
        let resource: AllocableBlock<F, A> = AllocableBlock::new(backend, name, size, family);
        let id = resource.allocable.resource_id();

        self.allocated
            .insert(resource.allocable.resource_id(), resource);

        self.allocated
            .get_mut(&id)
            .map(|resource| &mut resource.allocable)
            .ok_or(AllocationError::AllocationFailure)
    }

    pub fn recycle(&mut self, id: &Uuid) {
        let resource = self.allocated.remove(id);
        if let Some(resource) = resource {
            self.pool.insert(resource.allocable.family(), resource);
        }
    }

    pub fn recycle_all(&mut self) {
        self.allocated.drain().for_each(|(_, resource)| {
            self.pool.insert(resource.allocable.family(), resource);
        });
    }

    pub fn is_empty(&self) -> bool {
        self.allocated.is_empty()
    }
}
