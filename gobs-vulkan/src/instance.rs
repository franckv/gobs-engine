use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use anyhow::Result;
use ash::{ext::debug_utils, khr::surface, vk};
use raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::physical::PhysicalDevice;
use crate::queue::QueueFamily;
use crate::surface::Surface;
use crate::Wrap;

unsafe extern "system" fn debug_cb(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

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
            log::warn!(
                "{:?} [{} ({})] : {}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!(
                "{:?} [{} ({})] : {}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!(
                "{:?} [{} ({})] : {}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
            );
        }
        _ => {
            log::error!(
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
            log::info!("Create instance");

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

        Ok(Arc::new(Instance {
            instance,
            entry,
            surface_loader,
            debug_utils_loader,
            debug_call_back,
        }))
    }

    pub fn find_adapter(&self, surface: &Surface) -> PhysicalDevice {
        let mut p_devices = PhysicalDevice::enumerate(self);

        let idx = {
            let p_device = p_devices.iter().enumerate().find(|(_, p_device)| {
                let family = self.find_family(p_device, surface);
                let features = self.check_features(p_device);

                family.is_some() && features
            });

            match p_device {
                Some((idx, _)) => idx,
                None => panic!("No suitable device"),
            }
        };

        p_devices.remove(idx)
    }

    pub fn find_headless_adapter(&self) -> PhysicalDevice {
        let mut p_devices = PhysicalDevice::enumerate(self);

        let idx = {
            let p_device = p_devices.iter().enumerate().find(|(_, p_device)| {
                let family = self.find_headless_family(p_device);
                let features = self.check_features(p_device);

                family.is_some() && features
            });

            match p_device {
                Some((idx, _)) => idx,
                None => panic!("No suitable device"),
            }
        };

        p_devices.remove(idx)
    }

    fn check_features(&self, p_device: &PhysicalDevice) -> bool {
        let mut features11 = vk::PhysicalDeviceVulkan11Features::default();
        let mut features12 = vk::PhysicalDeviceVulkan12Features::default();
        let mut features13 = vk::PhysicalDeviceVulkan13Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut features11)
            .push_next(&mut features12)
            .push_next(&mut features13);

        unsafe {
            self.instance
                .get_physical_device_features2(p_device.raw(), &mut features);
        };

        let features10 = features.features;

        log::debug!(
            "Features: {:?},{:?},{:?},{:?}",
            features10,
            features11,
            features12,
            features13
        );

        features10.fill_mode_non_solid == 1
            && features12.buffer_device_address == 1
            && features12.descriptor_indexing == 1
            && features13.dynamic_rendering == 1
            && features13.synchronization2 == 1
    }

    pub fn find_family(&self, p_device: &PhysicalDevice, surface: &Surface) -> Option<QueueFamily> {
        let family = p_device
            .queue_families
            .iter()
            .find(|family| family.graphics_bit && surface.family_supported(&p_device, &family));

        match family {
            Some(family) => Some(family.clone()),
            None => None,
        }
    }

    pub fn find_headless_family(&self, p_device: &PhysicalDevice) -> Option<QueueFamily> {
        let family = p_device
            .queue_families
            .iter()
            .find(|family| family.graphics_bit);

        match family {
            Some(family) => Some(family.clone()),
            None => None,
        }
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
        log::debug!("Drop instance");
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}
