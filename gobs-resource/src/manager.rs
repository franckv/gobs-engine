use uuid::Uuid;

use gobs_core::{
    data::{anymap::AnyMap, objectmap::ObjectMap, registry::ObjectRegistry},
    logger,
};

use crate::resource::{
    Resource, ResourceError, ResourceHandle, ResourceLifetime, ResourceLoader, ResourceProperties,
    ResourceState, ResourceType,
};

#[derive(Default)]
pub struct ResourceRegistry {
    registry: ObjectRegistry<Uuid>,
    labels: ObjectMap<String, Uuid>,
}

pub struct ResourceData<'res, R: ResourceType> {
    pub data: &'res R::ResourceData,
    pub properties: &'res R::ResourceProperties,
}

pub struct ResourceDataMut<'res, R: ResourceType> {
    pub data: &'res mut R::ResourceData,
    pub properties: &'res mut R::ResourceProperties,
}

impl ResourceRegistry {
    pub fn add<R: ResourceType + 'static>(
        &mut self,
        properties: R::ResourceProperties,
        lifetime: ResourceLifetime,
    ) -> ResourceHandle<R> {
        self.add_or_replace(properties, lifetime, false)
    }

    fn add_or_replace<R: ResourceType + 'static>(
        &mut self,
        properties: R::ResourceProperties,
        lifetime: ResourceLifetime,
        replace: bool,
    ) -> ResourceHandle<R> {
        let name = properties.name().to_string();
        let resource = Resource::<R>::new(properties, lifetime);

        let handle = resource.handle;

        tracing::debug!(target: logger::RESOURCES, "New resource: {} ({}): {:?}", &name, std::any::type_name::<R>(), handle.id);

        self.registry.insert(handle.id, resource);
        if !replace && self.labels.insert::<R>(name.clone(), handle.id).is_some() {
            tracing::debug!(target: logger::RESOURCES, "Replace resource: {} ({}): {:?}", &name, std::any::type_name::<R>(), handle.id);
        }

        handle
    }

    pub fn remove<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> Option<Resource<R>> {
        self.registry.remove::<Resource<R>>(&handle.id)
    }

    /// Clone a resource and schedule old resource for deletion
    pub fn replace<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> ResourceHandle<R> {
        tracing::trace!(target: logger::RESOURCES, "Resource cloned: {:?}", handle);

        let old_resource = self.get_mut::<R>(handle);
        let properties = old_resource.properties.clone();
        let lifetime = old_resource.lifetime;
        old_resource.life = 0;
        old_resource.lifetime = ResourceLifetime::Transient;

        self.add_or_replace(properties, lifetime, true)
    }

    pub fn get_by_name<R: ResourceType + 'static>(&self, name: &str) -> Option<ResourceHandle<R>> {
        let id = self.labels.get::<R>(&name.to_string());

        id.map(|id| ResourceHandle::with_uuid(*id))
    }

    // TODO: return option
    pub fn get<R: ResourceType + 'static>(&self, handle: &ResourceHandle<R>) -> &Resource<R> {
        self.registry.get::<Resource<R>>(&handle.id).unwrap()
    }

    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> &mut Resource<R> {
        self.registry.get_mut::<Resource<R>>(&handle.id).unwrap()
    }

    pub fn values<R: ResourceType + 'static>(&self) -> impl Iterator<Item = &Resource<R>> {
        self.registry.values()
    }

    pub fn values_mut<R: ResourceType + 'static>(
        &mut self,
    ) -> impl Iterator<Item = &mut Resource<R>> {
        self.registry.values_mut()
    }
}

pub struct ResourceManager {
    frames_in_flight: usize,
    registry: ResourceRegistry,
    loader: AnyMap,
}

impl ResourceManager {
    pub fn new(frames_in_flight: usize) -> Self {
        Self {
            frames_in_flight,
            registry: ResourceRegistry::default(),
            loader: AnyMap::default(),
        }
    }

    pub fn add<R: ResourceType + 'static>(
        &mut self,
        properties: R::ResourceProperties,
        lifetime: ResourceLifetime,
    ) -> ResourceHandle<R> {
        self.registry.add(properties, lifetime)
    }

    pub fn remove<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> Option<Resource<R>> {
        self.registry.remove(handle)
    }

    pub fn replace<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> ResourceHandle<R> {
        self.registry.replace(handle)
    }

    pub fn get_by_name<R: ResourceType + 'static>(&self, name: &str) -> Option<ResourceHandle<R>> {
        self.registry.get_by_name(name)
    }

    pub fn get<R: ResourceType + 'static>(&self, handle: &ResourceHandle<R>) -> &Resource<R> {
        self.registry.get(handle)
    }

    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> &mut Resource<R> {
        self.registry.get_mut(handle)
    }

    pub fn values<R: ResourceType + 'static>(&self) -> impl Iterator<Item = &Resource<R>> {
        self.registry.values()
    }

    pub fn update<R: ResourceType + 'static>(&mut self) {
        tracing::trace!(target: logger::RESOURCES, "Update registry");

        let mut to_delete = vec![];

        for value in self.registry.values_mut() {
            tracing::trace!(target: logger::RESOURCES, "Registry Type={:?}, key={:?}, life={}, lifetime={:?}", std::any::type_name::<R>(), value.handle.id, value.life, value.lifetime);

            if value.lifetime == ResourceLifetime::Transient {
                value.life += 1;
                if value.life > self.frames_in_flight {
                    tracing::trace!(target: logger::RESOURCES, "Transient resource to be removed: {}", value.handle.id);
                    to_delete.push(value.handle);
                }
            }
        }

        let loader = self.loader.get_mut::<R::ResourceLoader>().unwrap();
        for handle in to_delete {
            if let Some(resource) = self.registry.remove(&handle) {
                loader.unload(resource);
            }
        }
    }

    pub fn register_resource<R: ResourceType + 'static>(&mut self, loader: R::ResourceLoader) {
        self.loader.insert(loader);
    }

    fn load_data<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
        parameter: &R::ResourceParameter,
    ) -> Result<(), ResourceError> {
        let resource = self.registry.get_mut(handle);

        if !resource.is_loaded(parameter) {
            let loader = self
                .loader
                .get_mut::<R::ResourceLoader>()
                .unwrap_or_else(|| {
                    panic!("Loader not registered: {:?}", std::any::type_name::<R>())
                });

            tracing::trace!(target: logger::RESOURCES, "Loading resource {:?}", handle);
            let data = loader.load(handle, parameter, &mut self.registry)?;

            let resource = self.get_mut::<R>(handle);

            resource
                .data
                .insert(parameter.clone(), ResourceState::Loaded(data));
        }

        Ok(())
    }

    pub fn get_data<R: ResourceType + 'static>(
        &'_ mut self,
        handle: &ResourceHandle<R>,
        parameter: R::ResourceParameter,
    ) -> Result<ResourceData<'_, R>, ResourceError> {
        self.load_data::<R>(handle, &parameter)?;

        let resource = self.get(handle);

        let data = match resource.data.get(&parameter) {
            Some(ResourceState::Loaded(data)) => data,
            _ => unreachable!(),
        };

        Ok(ResourceData {
            data,
            properties: &resource.properties,
        })
    }

    pub fn get_data_mut<R: ResourceType + 'static>(
        &'_ mut self,
        handle: &ResourceHandle<R>,
        parameter: R::ResourceParameter,
    ) -> Result<ResourceDataMut<'_, R>, ResourceError> {
        self.load_data::<R>(handle, &parameter)?;

        let resource = self.get_mut(handle);

        let data = match resource.data.get_mut(&parameter) {
            Some(ResourceState::Loaded(data)) => data,
            _ => unreachable!(),
        };

        Ok(ResourceDataMut {
            data,
            properties: &mut resource.properties,
        })
    }
}
