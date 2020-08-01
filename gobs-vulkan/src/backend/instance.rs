use std::default::Default;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Arc;

use ash::{self, vk};
use ash::extensions::ext::DebugReport;
use ash::extensions::khr::Surface as VkSurface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;

use crate::backend::physical::PhysicalDevice;
use crate::backend::queue::QueueFamily;
use crate::backend::surface::Surface;

unsafe extern "system" fn debug_cb(_: vk::DebugReportFlagsEXT,
                                   _: vk::DebugReportObjectTypeEXT,
                                   _: u64,
                                   _: usize,
                                   _: i32,
                                   _: *const std::os::raw::c_char,
                                   error: *const std::os::raw::c_char,
                                   _: *mut std::os::raw::c_void) -> vk::Bool32 {
    let msg = CStr::from_ptr(error).to_string_lossy();
    error!("DEBUG: {}", msg);
    vk::FALSE
}

pub struct Instance {
    pub(crate) instance: ash::Instance,
    pub(crate) entry: ash::Entry,
    pub(crate) surface_loader: VkSurface,
    debug_report_entry: DebugReport,
    debug_report: Option<vk::DebugReportCallbackEXT>
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
            DebugReport::name().as_ptr(),
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

        let entry = ash::Entry::new().unwrap();

        let instance: ash::Instance = unsafe {
            debug!("Create instance");


            entry.create_instance(&instance_info, None).unwrap()
        };

        let surface_loader =
            VkSurface::new(&entry, &instance);

        let debug_report_entry = DebugReport::new(&entry, &instance);

        let debug_report_info = vk::DebugReportCallbackCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
            p_next: ptr::null(),
            flags: vk::DebugReportFlagsEXT::WARNING |
                vk::DebugReportFlagsEXT::PERFORMANCE_WARNING |
                vk::DebugReportFlagsEXT::ERROR,
            pfn_callback: Some(debug_cb),
            p_user_data: ptr::null_mut(),
        };

        let debug_report = unsafe {
            debug!("Create debug report");

            Some(debug_report_entry.create_debug_report_callback(
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
                self.debug_report_entry.destroy_debug_report_callback(
                    self.debug_report.unwrap(), None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
