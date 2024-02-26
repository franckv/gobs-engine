use std::{ffi::CString, sync::Arc};

use ash::vk;

use crate::device::Device;

pub(crate) fn add_label(
    device: Arc<Device>,
    label: &str,
    object_type: vk::ObjectType,
    object_handle: u64,
) {
    let label = CString::new(label).unwrap();

    let name_info = vk::DebugUtilsObjectNameInfoEXT::builder()
        .object_type(object_type)
        .object_handle(object_handle)
        .object_name(&label);

    unsafe {
        device
            .instance()
            .debug_utils_loader
            .set_debug_utils_object_name(device.cloned().handle(), &name_info)
            .unwrap();
    }
}
