use ash::{Device, Entry, Instance};
use ash::version::V1_0;

pub mod buffer;
pub mod command;
pub mod descriptor;
pub mod device;
pub mod framebuffer;
pub mod image;
pub mod instance;
pub mod memory;
pub mod physical;
pub mod pipeline;
pub mod queue;
pub mod renderpass;
pub mod surface;
pub mod sync;
pub mod swapchain;

type Version = V1_0;
type VkDevice = Device<Version>;
type VkEntry = Entry<Version>;
type VkInstance = Instance<Version>;

trait Wrap<T> {
    fn raw(&self) -> T;
}
