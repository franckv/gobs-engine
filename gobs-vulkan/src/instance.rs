use std::borrow::Cow;
use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use anyhow::Result;
use ash::{ext::debug_utils, khr::surface, vk};
use raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::{
    feature::Features,
    physical::{PhysicalDevice, PhysicalDeviceType},
    queue::QueueFamily,
    surface::Surface,
};

unsafe extern "system" fn debug_cb(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    unsafe {
        let callback_data = *p_callback_data;
        let message_id_number: i32 = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
                tracing::warn!(
                    "{:?} [{} ({})] : {}",
                    message_type,
                    message_id_name,
                    &message_id_number.to_string(),
                    message,
                );
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
                tracing::info!(
                    "{:?} [{} ({})] : {}",
                    message_type,
                    message_id_name,
                    &message_id_number.to_string(),
                    message,
                );
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
                tracing::debug!(
                    "{:?} [{} ({})] : {}",
                    message_type,
                    message_id_name,
                    &message_id_number.to_string(),
                    message,
                );
            }
            _ => {
                tracing::error!(
                    "{:?} [{} ({})] : {}",
                    message_type,
                    message_id_name,
                    &message_id_number.to_string(),
                    message,
                );
                #[cfg(debug_assertions)]
                panic!("{}", message);
            }
        }

        vk::FALSE
    }
}

/// First object to create. Link to Vulkan runtime
pub struct Instance {
    pub(crate) instance: ash::Instance,
    pub(crate) entry: ash::Entry,
    pub(crate) surface_loader: surface::Instance,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    pub(crate) debug_utils_loader: debug_utils::Instance,
}

impl Instance {
    pub fn new(
        name: &str,
        version: u32,
        window: Option<&Window>,
        validation: bool,
    ) -> Result<Arc<Self>> {
        let app_name = CString::new(name).unwrap();

        let vk_version = vk::make_api_version(0, 1, 3, 0);

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(version)
            .engine_name(&app_name)
            .engine_version(version)
            .api_version(vk_version);

        let mut extensions = match window {
            Some(window) => {
                ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())
                    .unwrap()
                    .to_vec()
            }
            None => vec![ash::khr::surface::NAME.as_ptr()],
        };

        extensions.push(debug_utils::NAME.as_ptr());

        let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();

        let layers = if validation {
            vec![validation_layer.as_ptr()]
        } else {
            vec![]
        };

        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions);

        let entry = ash::Entry::linked();

        let instance: ash::Instance = unsafe {
            tracing::info!("Create instance");

            entry.create_instance(&instance_info, None).unwrap()
        };

        let surface_loader = surface::Instance::new(&entry, &instance);

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(debug_cb));

        let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        let instance = Instance {
            instance,
            entry,
            surface_loader,
            debug_utils_loader,
            debug_call_back,
        };

        Ok(Arc::new(instance))
    }

    pub fn find_adapter(
        &self,
        expected_features: &Features,
        _surface: Option<&Surface>,
    ) -> Option<PhysicalDevice> {
        let mut p_devices = PhysicalDevice::enumerate(self);
        let mut candidates = vec![];

        tracing::debug!(target: "init", "{} physical devices found", p_devices.len());

        for p_device in p_devices.drain(..) {
            if self.check_physical_device(&p_device, expected_features) {
                candidates.push(p_device);
            }
        }

        candidates.into_iter().max_by(|_, device2| {
            if device2.gpu_type == PhysicalDeviceType::DiscreteGpu {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    }

    fn check_physical_device(
        &self,
        p_device: &PhysicalDevice,
        expected_features: &Features,
    ) -> bool {
        tracing::debug!(target: "init", "Checking device: {:?}", p_device.name);

        tracing::debug!(target: "init", "Device type: {:?}", p_device.props.device_type);

        let vram = p_device
            .mem_props
            .memory_heaps_as_slice()
            .iter()
            .map(|heap| {
                if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
                    heap.size
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0);

        tracing::debug!(target: "init", "VRAM size: {}", vram);

        if p_device.props.api_version < vk::make_api_version(0, 1, 3, 0) {
            tracing::debug!(target: "init", "Reject: wrong version");
            return false;
        }

        let features = Features::from_device(self, p_device);

        if !features.check_features(expected_features) {
            tracing::debug!(target: "init", "Reject: missing features");
            return false;
        }

        tracing::debug!(target: "init", "Accepted");

        true
    }

    pub fn find_family(
        &self,
        p_device: &PhysicalDevice,
        surface: Option<&Surface>,
    ) -> (QueueFamily, QueueFamily) {
        let graphics_family = p_device.queue_families.iter().find(|family| match surface {
            Some(surface) => family.graphics_bit && surface.family_supported(p_device, family),
            None => family.graphics_bit,
        });

        let transfer_family = p_device
            .queue_families
            .iter()
            .find(|family| family.transfer_bits && !family.graphics_bit);

        let graphics_family = graphics_family.expect("Get graphics family").clone();
        let transfer_family = transfer_family.unwrap_or(&graphics_family).clone();

        (graphics_family, transfer_family)
    }

    pub fn raw(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn cloned(&self) -> ash::Instance {
        self.instance.clone()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        tracing::debug!(target: "memory", "Drop instance");
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}
