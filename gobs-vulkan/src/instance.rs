use std::borrow::Cow;
use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use ash::{ext::debug_utils, khr::surface, vk};
use gobs_core::logger;
use raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::error::VulkanError;
use crate::{
    feature::Features,
    physical::{PhysicalDevice, PhysicalDeviceType},
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
                tracing::warn!(target: logger::RENDER,
                    "{:?} [{} ({})] : {}",
                    message_type,
                    message_id_name,
                    &message_id_number.to_string(),
                    message,
                );
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
                tracing::info!(target: logger::RENDER,
                    "{:?} [{} ({})] : {}",
                    message_type,
                    message_id_name,
                    &message_id_number.to_string(),
                    message,
                );
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
                tracing::debug!(target: logger::RENDER,
                    "{:?} [{} ({})] : {}",
                    message_type,
                    message_id_name,
                    &message_id_number.to_string(),
                    message,
                );
            }
            _ => {
                tracing::error!(target: logger::RENDER,
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
    ) -> Result<Arc<Self>, VulkanError> {
        let app_name = CString::new(name)?;

        let vk_version = vk::make_api_version(0, 1, 3, 0);

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(version)
            .engine_name(&app_name)
            .engine_version(version)
            .api_version(vk_version);

        let mut extensions = match window {
            Some(window) => ash_window::enumerate_required_extensions(
                window
                    .display_handle()
                    .map_err(|_| VulkanError::InstanceCreateError)?
                    .as_raw(),
            )?
            .to_vec(),
            None => vec![ash::khr::surface::NAME.as_ptr()],
        };

        extensions.push(debug_utils::NAME.as_ptr());

        let validation_layer = CString::new("VK_LAYER_KHRONOS_validation")?;

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

        unsafe {
            tracing::debug!(
            target: logger::INIT,
                            "Available extensions: {:?}",
                            entry
                                .enumerate_instance_extension_properties(None)?
                                .iter()
                                .map(|ext| ext.extension_name_as_c_str().unwrap())
                                .collect::<Vec<&CStr>>()
                        );

            tracing::debug!(
            target: logger::INIT,
                            "Available layers: {:?}",
                            entry
                                .enumerate_instance_layer_properties()?
                                .iter()
                                .map(|layer| layer.layer_name_as_c_str().unwrap())
                                .collect::<Vec<&CStr>>()
                        );
            tracing::debug!(target: logger::INIT, "Enabled extensions {:?}", extensions.iter().map(|ext| CStr::from_ptr(*ext)).collect::<Vec<&CStr>>());
            tracing::debug!(target: logger::INIT, "Enabled layers {:?}", layers.iter().map(|ext| CStr::from_ptr(*ext)).collect::<Vec<&CStr>>());
        }

        tracing::info!(target: logger::INIT, "Create instance");
        let instance: ash::Instance = unsafe { entry.create_instance(&instance_info, None)? };

        tracing::debug!(target: logger::INIT, "Create surface loader");
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
        let debug_call_back =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None)? };

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

        tracing::debug!(target: logger::INIT, "{} physical devices found", p_devices.len());

        for p_device in p_devices.drain(..) {
            if p_device.check_features(self, expected_features) {
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

    pub fn raw(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn cloned(&self) -> ash::Instance {
        self.instance.clone()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        tracing::debug!(target: logger::MEMORY, "Drop instance");
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}
