use std::sync::Arc;

use crate::{device::Device, instance::Instance};

pub struct Context {
    pub device: Arc<Device>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        let instance = Instance::new(name, 1, None, true).expect("Failed to init Intance");

        let physical_device = instance.find_adapter(None).expect("Find suitable adapter");

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
