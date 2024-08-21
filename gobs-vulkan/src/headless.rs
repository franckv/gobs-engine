use std::sync::Arc;

use crate::{device::Device, instance::Instance};

pub struct Context {
    pub device: Arc<Device>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        let instance = Instance::new(name, 1, None, true).expect("Failed to init Intance");

        let physical_device = instance.find_headless_adapter();

        tracing::info!("Using adapter {}", physical_device.name);

        let queue_family = instance.find_headless_family(&physical_device).unwrap();

        let device = Device::new(instance.clone(), physical_device, &queue_family);

        Context { device }
    }
}
