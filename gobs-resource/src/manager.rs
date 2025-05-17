use std::collections::hash_map::Entry;

use uuid::Uuid;

use gobs_core::utils::{anymap::AnyMap, registry::ObjectRegistry};

use crate::resource::{
    Resource, ResourceHandle, ResourceLifetime, ResourceLoader, ResourceState, ResourceType,
};

pub struct ResourceManager {
    frames_in_flight: usize,
    registry: ObjectRegistry<Uuid>,
    loader: AnyMap,
}

impl ResourceManager {
    pub fn new(frames_in_flight: usize) -> Self {
        Self {
            frames_in_flight,
            registry: ObjectRegistry::default(),
            loader: AnyMap::default(),
        }
    }

    pub fn update<R: ResourceType + 'static>(&mut self) {
        tracing::trace!(target: "resources", "Update registry");

        let mut to_delete = vec![];

        for value in self.registry.values_mut::<Resource<R>>() {
            tracing::debug!(target: "resources", "Registry Type={:?}, key={:?}, life={}, lifetime={:?}", std::any::type_name::<R>(), value.handle.id, value.life, value.lifetime);

            if value.lifetime == ResourceLifetime::Transient {
                value.life += 1;
                if value.life > self.frames_in_flight {
                    tracing::debug!(target: "resources", "To be removed: {}", value.handle.id);
                    to_delete.push(value.handle);
                }
            }
        }

        for handle in to_delete {
            self.remove::<R>(&handle);
        }
    }

    pub fn register_resource<R: ResourceType + 'static>(&mut self, loader: R::ResourceLoader) {
        self.loader.insert(loader);
    }

    pub fn add<R: ResourceType + 'static>(
        &mut self,
        properties: R::ResourceProperties,
        lifetime: ResourceLifetime,
    ) -> ResourceHandle<R> {
        let resource = Resource::<R>::new(properties, lifetime);

        let handle = resource.handle;

        tracing::trace!(target: "resources", "New resource: {:?}", handle.id);

        self.registry.insert(handle.id, resource);

        handle
    }

    pub fn remove<R: ResourceType + 'static>(&mut self, handle: &ResourceHandle<R>) {
        self.registry.remove::<Resource<R>>(&handle.id);
    }

    pub fn replace<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> ResourceHandle<R> {
        tracing::trace!(target: "resources", "Resource cloned: {:?}", handle);

        let old_resource = self.get_mut::<R>(handle);
        let properties = old_resource.properties.clone();
        let lifetime = old_resource.lifetime;
        old_resource.life = 0;
        old_resource.lifetime = ResourceLifetime::Transient;

        self.add(properties, lifetime)
    }

    fn load_data<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
        parameter: &R::ResourceParameter,
    ) {
        let resource = self.registry.get_mut::<Resource<R>>(&handle.id).unwrap();

        match resource.data.entry(parameter.clone()) {
            Entry::Occupied(mut e) => {
                if let ResourceState::Unloaded = e.get() {
                    tracing::trace!(target: "resources", "Loading resource {:?}", handle);
                    let loader = self.loader.get_mut::<R::ResourceLoader>().unwrap();
                    let data = loader.load(&mut resource.properties, parameter);
                    e.insert(ResourceState::Loaded(data));
                }
            }
            Entry::Vacant(e) => {
                tracing::trace!(target: "resources", "Loading resource {:?}", handle);
                let loader = self.loader.get_mut::<R::ResourceLoader>().unwrap();
                let data = loader.load(&mut resource.properties, parameter);
                e.insert(ResourceState::Loaded(data));
            }
        }
    }

    pub fn get_data<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
        parameter: R::ResourceParameter,
    ) -> &R::ResourceData {
        self.load_data::<R>(handle, &parameter);

        let resource = self.registry.get_mut::<Resource<R>>(&handle.id).unwrap();
        match &resource.data.get(&parameter) {
            Some(ResourceState::Loaded(data)) => data,
            _ => unreachable!(),
        }
    }

    pub fn get<R: ResourceType + 'static>(&mut self, handle: &ResourceHandle<R>) -> &Resource<R> {
        self.registry.get_mut::<Resource<R>>(&handle.id).unwrap()
    }

    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> &mut Resource<R> {
        self.registry.get_mut::<Resource<R>>(&handle.id).unwrap()
    }
}
