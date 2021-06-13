use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use ash::{self, vk};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface as VkSurface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;
#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;

use crate::backend::physical::PhysicalDevice;
use crate::backend::queue::QueueFamily;
use crate::backend::surface::Surface;

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

    error!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}

/// First object to create. Link to Vulkan runtime
pub struct Instance {
    pub(crate) instance: ash::Instance,
    pub(crate) entry: ash::Entry,
    pub(crate) surface_loader: VkSurface,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    debug_utils_loader: DebugUtils,
}

impl Instance {
    pub fn new(name: &str, version: u32) -> Arc<Self> {
        let app_name = CString::new(name).unwrap();

        let vk_version = vk::make_version(1, 0, 0);

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(version)
            .engine_name(&app_name)
            .engine_version(version)
            .api_version(vk_version);

        let extensions = [
            VkSurface::name().as_ptr(),
            #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
            XlibSurface::name().as_ptr(),
            #[cfg(target_os = "windows")]
            Win32Surface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ];

        //let validation = CString::new("VK_LAYER_LUNARG_standard_validation").unwrap();
        let validation = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
        //let debug_validation = CString::new("VK_LAYER_LUNARG_api_dump").unwrap();

        let layers = [
            validation.as_ptr(),
            //debug_validation.as_ptr()
        ];

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions);

        let entry = unsafe {ash::Entry::new().unwrap()};

        let instance: ash::Instance = unsafe {
            debug!("Create instance");


            entry.create_instance(&instance_info, None).unwrap()
        };

        let surface_loader =
            VkSurface::new(&entry, &instance);

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(debug_cb));

        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        let debug_call_back = unsafe {
            debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .unwrap()
        };
        
        Arc::new(Instance {
            instance,
            entry,
            surface_loader,
            debug_utils_loader,
            debug_call_back,
        })
    }

    pub fn find_adapter(&self, surface: &Surface) -> PhysicalDevice {
        let mut p_devices = PhysicalDevice::enumerate(self);

        let idx = {
            let p_device = p_devices.iter().enumerate().find(|(_, p_device)| {
                let family = self.find_family(&p_device, surface);

                family.is_some()
            });

            match p_device {
                Some((idx, _)) => idx,
                None => panic!("No suitable device")
            }
        };

        p_devices.remove(idx)
    }

    pub fn find_family(&self, p_device: &PhysicalDevice,
                       surface: &Surface) -> Option<QueueFamily> {
        let family = p_device.queue_families.iter().find(|family| {
            family.graphics_bit &&
                surface.family_supported(&p_device, &family)
        });

        match family {
            Some(family) => Some(family.clone()),
            None => None
        }
    }

    pub(crate) fn raw(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        trace!("Drop instance");
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}
