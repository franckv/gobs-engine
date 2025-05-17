use std::collections::HashMap;
use std::hash::Hash;

use uuid::Uuid;

pub enum ResourceState<R: ResourceType> {
    Unloaded,
    Loading,
    Loaded(R::ResourceData),
}

pub type ResourceHandle = Uuid;

pub trait ResourceType {
    type ResourceData;
    type ResourceProperties: Clone;
    type ResourceParameter: Clone + Hash + Eq;
    type ResourceLoader: ResourceLoader<Self>
    where
        Self: Sized;
}

pub struct Resource<R: ResourceType> {
    pub id: ResourceHandle,
    pub properties: R::ResourceProperties,
    pub(crate) data: HashMap<R::ResourceParameter, ResourceState<R>>,
}

impl<R: ResourceType> Resource<R> {
    pub(crate) fn new(properties: R::ResourceProperties) -> Self {
        Self {
            id: Uuid::new_v4(),
            properties,
            data: HashMap::new(),
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
