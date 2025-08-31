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

pub trait Allocable<D, F: ResourceFamily> {
    fn resource_id(&self) -> Uuid;
    fn family(&self) -> F;
    fn resource_size(&self) -> usize;
    fn allocate(device: &D, name: &str, size: usize, family: F) -> Self;
}

pub struct AllocableBlock<D, F, A>
where
    F: ResourceFamily,
    A: Allocable<D, F>,
{
    allocable: A,
    block_allocator: BuddyAllocator,
    device: PhantomData<D>,
    family: PhantomData<F>,
}

impl<D, F: ResourceFamily, A: Allocable<D, F>> AllocableBlock<D, F, A> {
    pub fn new(device: &D, name: &str, size: usize, family: F) -> Self {
        Self {
            allocable: A::allocate(device, name, size, family),
            block_allocator: BuddyAllocator::new(size, 5).unwrap(),
            device: std::marker::PhantomData,
            family: std::marker::PhantomData,
        }
    }

    pub fn max_size(&self) -> usize {
        self.block_allocator.max_available_size()
    }
}

pub struct Allocator<D, F, A>
where
    F: ResourceFamily,
    A: Allocable<D, F>,
{
    pub pool: ObjectPool<F, AllocableBlock<D, F, A>>,
    pub allocated: HashMap<Uuid, AllocableBlock<D, F, A>>,
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
            allocated: HashMap::new(),
            device: PhantomData,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn allocate(
        &mut self,
        device: &D,
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
        let resource: AllocableBlock<D, F, A> = AllocableBlock::new(device, name, size, family);
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
