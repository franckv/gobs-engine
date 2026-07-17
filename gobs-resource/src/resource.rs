use std::{fmt::Debug, marker::PhantomData};

use serde::Serialize;
use thiserror::Error;

use gobs_core::memory::allocator::AllocationError;

use crate::{
    load::LoadingError,
    manager::{ResourceId, ResourceRegistry},
};

#[allow(unused)]
pub enum ResourceState<R: ResourceType> {
    Unloaded,
    Loading,
    Loaded(R::ResourceData),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ResourceLifetime {
    Static,
    Transient,
}

#[derive(PartialEq, Serialize)]
pub struct ResourceHandle<R: ResourceType> {
    pub id: ResourceId,
    pub(crate) ty: PhantomData<R>,
}

impl<R: ResourceType> Clone for ResourceHandle<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: ResourceType> Copy for ResourceHandle<R> {}

impl<R: ResourceType> Debug for ResourceHandle<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceHandle")
            .field("id", &self.id)
            .finish()
    }
}

impl<R: ResourceType> ResourceHandle<R> {
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            ty: PhantomData,
        }
    }
}

pub trait ResourceType: Copy + Debug {
    type ResourceData;
    type ResourceBackend<'a>: ?Sized;
    type ResourceProperties: ResourceProperties + Clone;
    type ResourceLoader: ResourceLoader<Self>;
}

pub struct Resource<R: ResourceType> {
    pub handle: ResourceHandle<R>,
    pub properties: R::ResourceProperties,
    pub(crate) data: ResourceState<R>,
    pub lifetime: ResourceLifetime,
    pub life: usize,
}

impl<R: ResourceType> Resource<R> {
    pub(crate) fn new(
        handle: ResourceHandle<R>,
        properties: R::ResourceProperties,
        lifetime: ResourceLifetime,
    ) -> Self {
        Self {
            handle,
            properties,
            data: ResourceState::Unloaded,
            lifetime,
            life: 0,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub(crate) fn is_loaded(&self) -> bool {
        matches!(self.data, ResourceState::Loaded(_))
    }
}

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("resource loading error")]
    ResourceLoadError(#[from] LoadingError),
    #[error("allocation error")]
    AllocationError(#[from] AllocationError),
    #[error("invalid data")]
    InvalidData,
}

pub trait ResourceProperties {
    fn name(&self) -> &str;
}

pub trait ResourceLoader<R: ResourceType> {
    fn load<'a>(
        &mut self,
        backend: &mut R::ResourceBackend<'a>,
        handle: &ResourceHandle<R>,
        resource_registry: &mut ResourceRegistry,
    ) -> Result<R::ResourceData, ResourceError>;

    fn unload(&mut self, resource: Resource<R>);
}
