use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use serde::Serialize;
use uuid::Uuid;

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

// pub type ResourceHandle = Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct ResourceHandle<R: ResourceType> {
    pub id: Uuid,
    pub(crate) ty: PhantomData<R>,
}

impl<R: ResourceType> ResourceHandle<R> {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            ty: PhantomData,
        }
    }
}

impl<R: ResourceType> Default for ResourceHandle<R> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ResourceType: Copy + Debug {
    type ResourceData;
    type ResourceProperties: Clone;
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
    pub(crate) fn new(properties: R::ResourceProperties, lifetime: ResourceLifetime) -> Self {
        Self {
            handle: ResourceHandle::new(),
            properties,
            data: HashMap::new(),
            lifetime,
            life: 0,
        }
    }
}

pub trait ResourceLoader<R: ResourceType> {
    fn load(
        &mut self,
        properties: &mut R::ResourceProperties,
        parameter: &R::ResourceParameter,
    ) -> R::ResourceData;
}
