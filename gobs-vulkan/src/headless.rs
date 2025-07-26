use std::sync::Arc;

use gobs_core::logger;

use crate::{device::Device, feature::Features, instance::Instance};

pub struct Context {
    pub device: Arc<Device>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        let instance = Instance::new(name, 1, None, false).expect("Failed to init Instance");

        let expected_features = Features::default()
            .fill_mode_non_solid()
            .buffer_device_address()
            .descriptor_indexing()
            .dynamic_rendering()
            .synchronization2();

        let physical_device = instance
            .find_adapter(&expected_features, None)
            .expect("Find suitable adapter");

        tracing::info!(target: logger::RENDER, "Using adapter {}", physical_device.name);

        let device =
            Device::new(instance.clone(), physical_device, None).expect("Failed to init Device");

        Context { device }
    }
}
