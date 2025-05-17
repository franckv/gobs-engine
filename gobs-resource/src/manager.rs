use std::collections::hash_map::Entry;

use gobs_core::utils::{anymap::AnyMap, registry::ObjectRegistry};

use crate::resource::{Resource, ResourceHandle, ResourceLoader, ResourceState, ResourceType};

pub struct ResourceManager {
    registry: ObjectRegistry,
    loader: AnyMap,
}

impl ResourceManager {
    pub fn register_loader<R: ResourceType + 'static>(&mut self, loader: R::ResourceLoader) {
        self.loader.insert(loader);
    }

    pub fn add<R: ResourceType + 'static>(
        &mut self,
        properties: R::ResourceProperties,
    ) -> ResourceHandle {
        let resource = Resource::<R>::new(properties);

        let handle = resource.id;

        tracing::debug!(target: "resources", "New resource: {:?}", handle);

        self.registry.insert(handle, resource);

        handle
    }

    pub fn clone<R: ResourceType + 'static>(&mut self, handle: &ResourceHandle) -> ResourceHandle {
        tracing::debug!(target: "resources", "Resource cloned: {}", handle);

        let old_resource = self.get::<R>(handle);
        let properties = old_resource.properties.clone();

        self.add::<R>(properties)
    }

    fn load_data<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle,
        parameter: &R::ResourceParameter,
    ) {
        let resource = self.registry.get_mut::<Resource<R>>(handle).unwrap();

        match resource.data.entry(parameter.clone()) {
            Entry::Occupied(mut e) => {
                if let ResourceState::Unloaded = e.get() {
                    tracing::debug!(target: "resources", "Loading resource {}", handle);
                    let loader = self.loader.get_mut::<R::ResourceLoader>().unwrap();
                    let data = loader.load(&mut resource.properties, parameter);
                    e.insert(ResourceState::Loaded(data));
                }
            }
            Entry::Vacant(e) => {
                tracing::debug!(target: "resources", "Loading resource {}", handle);
                let loader = self.loader.get_mut::<R::ResourceLoader>().unwrap();
                let data = loader.load(&mut resource.properties, parameter);
                e.insert(ResourceState::Loaded(data));
            }
        }
    }

    pub fn get_data<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle,
        parameter: R::ResourceParameter,
    ) -> &R::ResourceData {
        self.load_data::<R>(handle, &parameter);

        let resource = self.registry.get_mut::<Resource<R>>(handle).unwrap();
        match &resource.data.get(&parameter) {
            Some(ResourceState::Loaded(data)) => data,
            _ => unreachable!(),
        }
    }

    pub fn get<R: ResourceType + 'static>(&mut self, handle: &ResourceHandle) -> &Resource<R> {
        self.registry.get_mut::<Resource<R>>(handle).unwrap()
    }

    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle,
    ) -> &mut Resource<R> {
        self.registry.get_mut::<Resource<R>>(handle).unwrap()
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            registry: ObjectRegistry::new(),
            loader: AnyMap::new(),
        }
    }
}
