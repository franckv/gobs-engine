use std::{ffi::CString, sync::Arc};

use ash::vk::{self, Handle};

use crate::device::Device;

pub(crate) fn add_label<T: Handle>(device: Arc<Device>, label: &str, object_handle: T) {
    let label = CString::new(label).unwrap();

    let name_info = vk::DebugUtilsObjectNameInfoEXT::default()
        .object_handle(object_handle)
        .object_name(&label);

    unsafe {
        device
            .debug_utils_device
            .set_debug_utils_object_name(&name_info)
            .unwrap();
    }
}
