use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use thiserror::Error;
use uuid::Uuid;

use crate::data::pool::ObjectPool;
use crate::logger;

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

pub struct Allocator<D, F, A>
where
    F: ResourceFamily,
    A: Allocable<D, F>,
{
    pub pool: ObjectPool<F, A>,
    pub allocated: HashMap<Uuid, A>,
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

    #[tracing::instrument(target = "memory", skip_all, level = "trace")]
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
                && resource.resource_size() >= size
            {
                tracing::debug!(
                    target: logger::RENDER,
                    "Reuse resource {:?}, {} ({})",
                    family,
                    size,
                    self.pool.get(&family).unwrap().len()
                );

                let id = resource.resource_id();
                self.allocated.insert(id, resource);

                return self
                    .allocated
                    .get_mut(&id)
                    .ok_or(AllocationError::AllocationFailure);
            }
        }

        tracing::debug!(target: logger::MEMORY, "Allocate new resource {:?}, {}", family, size);
        let resource = A::allocate(device, name, size, family);
        let id = resource.resource_id();

        self.allocated.insert(resource.resource_id(), resource);

        self.allocated
            .get_mut(&id)
            .ok_or(AllocationError::AllocationFailure)
    }

    #[tracing::instrument(target = "memory", skip_all, level = "trace")]
    pub fn recycle(&mut self, id: &Uuid) {
        let resource = self.allocated.remove(id);
        if let Some(resource) = resource {
            self.pool.insert(resource.family(), resource);
        }
    }
}
