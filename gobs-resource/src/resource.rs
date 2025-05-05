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
    type ResourceLoader: ResourceLoader<Self>
    where
        Self: Sized;
}

pub struct Resource<R: ResourceType> {
    pub id: ResourceHandle,
    pub properties: R::ResourceProperties,
    pub(crate) data: ResourceState<R>,
}

impl<R: ResourceType> Resource<R> {
    pub(crate) fn new(properties: R::ResourceProperties) -> Self {
        Self {
            id: Uuid::new_v4(),
            properties,
            data: ResourceState::Unloaded,
        }
    }
}

pub trait ResourceLoader<R: ResourceType> {
    fn load(&self, properties: &mut R::ResourceProperties) -> R::ResourceData;
}
