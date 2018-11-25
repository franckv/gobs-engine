use std::default::Default;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Arc;

use ash::{self, vk};
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::extensions::{DebugReport, Surface as VkSurface};
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::XlibSurface;

use super::{VkEntry, VkInstance};

use backend::physical::PhysicalDevice;
use backend::queue::QueueFamily;
use backend::surface::Surface;

unsafe extern "system" fn debug_cb(_: vk::DebugReportFlagsEXT,
                                   _: vk::DebugReportObjectTypeEXT,
                                   _: u64,
                                   _: usize,
                                   _: i32,
                                   _: *const vk::types::c_char,
                                   error: *const vk::types::c_char,
                                   _: *mut vk::types::c_void) -> vk::Bool32 {
    let msg = CStr::from_ptr(error).to_string_lossy();
    error!("DEBUG: {}", msg);
    vk::VK_FALSE
}

pub struct Instance {
    pub(crate) instance: VkInstance,
    pub(crate) entry: VkEntry,
    pub(crate) surface_loader: VkSurface,
    debug_report_entry: DebugReport,
    debug_report: Option<vk::DebugReportCallbackEXT>
}

impl Instance {
    pub fn new(name: &str, version: u32) -> Arc<Self> {
        let cname = CString::new(name).unwrap();
        let pname = cname.as_ptr();

        let vk_version = ash::vk_make_version
        !(1, 0, 36);

        let app_info = vk::ApplicationInfo {
            p_application_name: pname,
            s_type: vk::StructureType::ApplicationInfo,
            p_next: ptr::null(),
            application_version: version,
            p_engine_name: pname,
            engine_version: version,
            api_version: vk_version,
        };

        let extensions = [
            VkSurface::name().as_ptr(),
            #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
                XlibSurface::name().as_ptr(),
            DebugReport::name().as_ptr(),
        ];

        let validation =
            CString::new("VK_LAYER_LUNARG_standard_validation").unwrap();

        let layers = [
            validation.as_ptr()
        ];

        let instance_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::InstanceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            p_application_info: &app_info,
            pp_enabled_layer_names: layers.as_ptr(),
            enabled_layer_count: layers.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
        };

        let entry: VkEntry = VkEntry::new().unwrap();

        let instance: VkInstance = unsafe {
            debug!("Create instance");

            entry.create_instance(&instance_info, None).unwrap()
        };

        let surface_loader =
            VkSurface::new(&entry, &instance).unwrap();

        let debug_report_entry = DebugReport::new(&entry, &instance).unwrap();

        let debug_report_info = vk::DebugReportCallbackCreateInfoEXT {
            s_type: vk::StructureType::DebugReportCallbackCreateInfoExt,
            p_next: ptr::null(),
            flags: vk::DEBUG_REPORT_WARNING_BIT_EXT |
                vk::DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT |
                vk::DEBUG_REPORT_ERROR_BIT_EXT,
            pfn_callback: debug_cb,
            p_user_data: ptr::null_mut(),
        };

        let debug_report = unsafe {
            debug!("Create debug report");

            Some(debug_report_entry.create_debug_report_callback_ext(
                &debug_report_info, None).unwrap())
        };

        Arc::new(Instance {
            instance,
            entry,
            surface_loader,
            debug_report_entry,
            debug_report,
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
}

impl Drop for Instance {
    fn drop(&mut self) {
        trace!("Drop instance");
        unsafe {
            if self.debug_report.is_some() {
                self.debug_report_entry.destroy_debug_report_callback_ext(
                    self.debug_report.unwrap(), None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
