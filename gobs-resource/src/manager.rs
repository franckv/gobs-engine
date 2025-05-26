use uuid::Uuid;

use gobs_core::utils::{anymap::AnyMap, registry::ObjectRegistry};

use crate::resource::{
    Resource, ResourceHandle, ResourceLifetime, ResourceLoader, ResourceState, ResourceType,
};

#[derive(Default)]
pub struct ResourceRegistry {
    registry: ObjectRegistry<Uuid>,
}

impl ResourceRegistry {
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

    pub fn remove<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> Option<Resource<R>> {
        self.registry.remove::<Resource<R>>(&handle.id)
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

    pub fn get<R: ResourceType + 'static>(&self, handle: &ResourceHandle<R>) -> &Resource<R> {
        self.registry.get::<Resource<R>>(&handle.id).unwrap()
    }

    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> &mut Resource<R> {
        self.registry.get_mut::<Resource<R>>(&handle.id).unwrap()
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

    pub fn get<R: ResourceType + 'static>(&mut self, handle: &ResourceHandle<R>) -> &Resource<R> {
        self.registry.get(handle)
    }

    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> &mut Resource<R> {
        self.registry.get_mut(handle)
    }

    pub fn update<R: ResourceType + 'static>(&mut self) {
        tracing::trace!(target: "resources", "Update registry");

        let mut to_delete = vec![];

        for value in self.registry.values_mut() {
            tracing::trace!(target: "resources", "Registry Type={:?}, key={:?}, life={}, lifetime={:?}", std::any::type_name::<R>(), value.handle.id, value.life, value.lifetime);

            if value.lifetime == ResourceLifetime::Transient {
                value.life += 1;
                if value.life > self.frames_in_flight {
                    tracing::trace!(target: "resources", "Transient resource to be removed: {}", value.handle.id);
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
    ) {
        let resource = self.registry.get_mut(handle);

        if !resource.is_loaded(parameter) {
            let loader = self
                .loader
                .get_mut::<R::ResourceLoader>()
                .unwrap_or_else(|| {
                    panic!("Loader not registered: {:?}", std::any::type_name::<R>())
                });

            tracing::trace!(target: "resources", "Loading resource {:?}", handle);
            let data = loader.load(handle, parameter, &mut self.registry);

            let resource = self
                .registry
                .registry
                .get_mut::<Resource<R>>(&handle.id)
                .unwrap();

            resource
                .data
                .insert(parameter.clone(), ResourceState::Loaded(data));
        }
    }

    pub fn get_data<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
        parameter: R::ResourceParameter,
    ) -> &R::ResourceData {
        self.load_data::<R>(handle, &parameter);

        let resource = self
            .registry
            .registry
            .get_mut::<Resource<R>>(&handle.id)
            .unwrap();
        match &resource.data.get(&parameter) {
            Some(ResourceState::Loaded(data)) => data,
            _ => unreachable!(),
        }
    }
}
