use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use serde::Serialize;
use thiserror::Error;

use gobs_core::memory::allocator::AllocationError;

use crate::{
    load::LoadingError,
    manager::{ResourceId, ResourceRegistry},
};

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

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct ResourceHandle<R: ResourceType> {
    pub id: ResourceId,
    pub(crate) ty: PhantomData<R>,
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
    type ResourceProperties: ResourceProperties + Clone;
    type ResourceParameter: Clone + Hash + Eq;
    type ResourceLoader: ResourceLoader<Self>
    where
        Self: Sized;
}

pub struct Resource<R: ResourceType> {
    pub handle: ResourceHandle<R>,
    pub properties: R::ResourceProperties,
    pub(crate) data: HashMap<R::ResourceParameter, ResourceState<R>>,
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
            data: HashMap::new(),
            lifetime,
            life: 0,
        }
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub(crate) fn is_loaded(&self, parameter: &R::ResourceParameter) -> bool {
        matches!(self.data.get(parameter), Some(ResourceState::Loaded(_)))
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
    fn load(
        &mut self,
        handle: &ResourceHandle<R>,
        parameter: &R::ResourceParameter,
        resource_registry: &mut ResourceRegistry,
    ) -> Result<R::ResourceData, ResourceError>;

    fn unload(&mut self, resource: Resource<R>);
}
