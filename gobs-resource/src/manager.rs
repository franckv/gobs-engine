use gobs_core::{
    data::{
        anymap::AnyMap,
        object_arena::{Key, ObjectArena},
        objectmap::ObjectMap,
    },
    logger,
};

use crate::resource::{
    Resource, ResourceError, ResourceHandle, ResourceLifetime, ResourceLoader, ResourceProperties,
    ResourceState, ResourceType,
};

pub type ResourceId = Key;

#[derive(Default)]
pub struct ResourceRegistry {
    registry: ObjectArena,
    labels: ObjectMap<String, ResourceId>,
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
        tag: bool,
    ) -> ResourceHandle<R> {
        let key = self.registry.insert_with_key(|key| {
            Resource::<R>::new(ResourceHandle::new(key), properties, lifetime)
        });

        let resource: &Resource<R> = self.registry.get(key).unwrap();
        tracing::debug!(target: logger::RESOURCES, "New resource: {} ({}): {:?}", &resource.properties.name(), std::any::type_name::<R>(), resource.handle.id);

        if tag {
            self.labels
                .insert::<R>(resource.properties.name().to_string(), key);
        }

        resource.handle
    }

    pub fn remove<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> Option<Resource<R>> {
        self.registry.remove::<Resource<R>>(handle.id)
    }

    pub fn schedule_removal<R: ResourceType + 'static>(&mut self, handle: &ResourceHandle<R>) {
        let resource = self.get_mut(handle);
        resource.life = 0;
        resource.lifetime = ResourceLifetime::Transient;
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

        let key = self.registry.insert_with_key(|key| {
            Resource::<R>::new(ResourceHandle::new(key), properties, lifetime)
        });

        let resource: &Resource<R> = self.registry.get(key).unwrap();
        tracing::debug!(target: logger::RESOURCES, "New resource: {} ({}): {:?}", &resource.properties.name(), std::any::type_name::<R>(), resource.handle.id);

        resource.handle
    }

    pub fn get_by_name<R: ResourceType + 'static>(&self, name: &str) -> Option<ResourceHandle<R>> {
        let id = self.labels.get::<R>(&name.to_string());

        if id.is_none() {
            tracing::warn!(target: logger::RESOURCES, "Resource not found: {}", name);
        }

        id.map(|id| ResourceHandle::new(*id))
    }

    // TODO: return option
    pub fn get<R: ResourceType + 'static>(&self, handle: &ResourceHandle<R>) -> &Resource<R> {
        self.registry.get::<Resource<R>>(handle.id).unwrap()
    }

    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> &mut Resource<R> {
        self.registry.get_mut::<Resource<R>>(handle.id).unwrap()
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn add<R: ResourceType + 'static>(
        &mut self,
        properties: R::ResourceProperties,
        lifetime: ResourceLifetime,
        tag: bool,
    ) -> ResourceHandle<R> {
        self.registry.add(properties, lifetime, tag)
    }

    pub fn remove<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> Option<Resource<R>> {
        self.registry.remove(handle)
    }

    pub fn schedule_removal<R: ResourceType + 'static>(&mut self, handle: &ResourceHandle<R>) {
        self.registry.schedule_removal(handle);
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn replace<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> ResourceHandle<R> {
        self.registry.replace(handle)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn get_by_name<R: ResourceType + 'static>(&self, name: &str) -> Option<ResourceHandle<R>> {
        self.registry.get_by_name(name)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn get<R: ResourceType + 'static>(&self, handle: &ResourceHandle<R>) -> &Resource<R> {
        self.registry.get(handle)
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn get_mut<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
    ) -> &mut Resource<R> {
        self.registry.get_mut(handle)
    }

    pub fn values<R: ResourceType + 'static>(&self) -> impl Iterator<Item = &Resource<R>> {
        self.registry.values()
    }

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    pub fn update<R: ResourceType + 'static>(&mut self) {
        tracing::trace!(target: logger::RESOURCES, "Update registry");

        let mut to_delete = vec![];

        for value in self.registry.values_mut() {
            tracing::trace!(target: logger::RESOURCES, "Registry Type={:?}, key={:?}, life={}, lifetime={:?}", std::any::type_name::<R>(), value.handle.id, value.life, value.lifetime);

            if value.lifetime == ResourceLifetime::Transient {
                value.life += 1;
                if value.life > self.frames_in_flight {
                    tracing::trace!(target: logger::RESOURCES, "Transient resource to be removed: {:?}", value.handle.id);
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
    fn load_data<R: ResourceType + 'static>(
        &mut self,
        handle: &ResourceHandle<R>,
        parameter: &R::ResourceParameter,
    ) -> Result<(), ResourceError> {
        let resource = self.get_mut(handle);

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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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

    #[tracing::instrument(target = "profile", skip_all, level = "trace")]
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use slotmap::SlotMap;
    use tracing::Level;
    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::format::FmtSpan};

    use gobs_core::{logger, utils::timer::Timer};

    use crate::{
        manager::ResourceManager,
        resource::{
            ResourceError, ResourceLifetime, ResourceLoader, ResourceProperties, ResourceType,
        },
    };

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Dummy;

    impl ResourceType for Dummy {
        type ResourceData = DummyData;
        type ResourceProperties = DummyProperties;
        type ResourceParameter = ();
        type ResourceLoader = DummyLoader;
    }

    #[derive(Clone, Debug)]
    pub struct DummyProperties {
        pub name: String,
    }

    impl ResourceProperties for DummyProperties {
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[derive(Clone)]
    pub struct DummyData {}

    pub struct DummyLoader {}

    impl ResourceLoader<Dummy> for DummyLoader {
        fn load(
            &mut self,
            _handle: &crate::resource::ResourceHandle<Dummy>,
            _parameter: &<Dummy as ResourceType>::ResourceParameter,
            _resource_registry: &mut super::ResourceRegistry,
        ) -> Result<DummyData, ResourceError> {
            Ok(DummyData {})
        }

        fn unload(&mut self, _resource: crate::resource::Resource<Dummy>) {}
    }

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn bench_insert() {
        setup();

        let mut resource_manager = ResourceManager::new(2);
        resource_manager.register_resource::<Dummy>(DummyLoader {});

        let props = DummyProperties {
            name: "dummy".to_string(),
        };

        let mut timer = Timer::new();

        let mut keys = vec![];
        for _ in 0..10000 {
            keys.push(resource_manager.add::<Dummy>(
                props.clone(),
                ResourceLifetime::Static,
                false,
            ));
        }
        tracing::debug!(target: logger::PROFILE, "insert resource: {}", 1000. * timer.delta());

        for handle in keys {
            let _d = resource_manager.get_data(&handle, ());
        }
        tracing::debug!(target: logger::PROFILE, "get resource: {}", 1000. * timer.delta());

        let mut h = HashMap::new();
        for i in 0..10000 {
            h.insert(i, i);
        }
        tracing::debug!(target: logger::PROFILE, "insert hashmap: {}", 1000. * timer.delta());

        for i in 0..10000 {
            let _v = h.get(&i);
        }
        tracing::debug!(target: logger::PROFILE, "get hashmap: {}", 1000. * timer.delta());

        let mut v = Vec::new();
        for i in 0..10000 {
            v.push(i);
        }
        tracing::debug!(target: logger::PROFILE, "insert vec: {}", 1000. * timer.delta());

        for key in &v {
            let _d = v.get(*key);
        }
        tracing::debug!(target: logger::PROFILE, "get vec: {}", 1000. * timer.delta());

        let mut keys = vec![];
        let mut m = SlotMap::new();
        for i in 0..10000 {
            keys.push(m.insert(i));
        }
        tracing::debug!(target: logger::PROFILE, "insert slotmap: {}", 1000. * timer.delta());

        for key in &keys {
            let _d = m.get(*key);
        }
        tracing::debug!(target: logger::PROFILE, "get slotmap: {}", 1000. * timer.delta());
    }
}
