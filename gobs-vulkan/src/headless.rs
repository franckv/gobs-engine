use std::sync::Arc;

use crate::{device::Device, feature::Features, instance::Instance};

pub struct Context {
    pub device: Arc<Device>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        let instance = Instance::new(name, 1, None, true).expect("Failed to init Intance");

        let expected_features = Features::default()
            .fill_mode_non_solid()
            .buffer_device_address()
            .descriptor_indexing()
            .dynamic_rendering()
            .synchronization2();

        let physical_device = instance
            .find_adapter(&expected_features, None)
            .expect("Find suitable adapter");

        tracing::info!("Using adapter {}", physical_device.name);

        let (graphics_family, transfer_family) = instance.find_family(&physical_device, None);

        let device = Device::new(
            instance.clone(),
            physical_device,
            &graphics_family,
            &transfer_family,
        );

        Context { device }
    }
}
