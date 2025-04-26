pub mod alloc;
pub mod buffers;
pub mod command;
pub(crate) mod debug;
pub mod descriptor;
pub mod device;
pub mod error;
pub mod feature;
pub mod framebuffer;
pub mod images;
pub mod instance;
pub mod memory;
pub mod physical;
pub mod pipelines;
pub mod query;
pub mod queue;
pub mod renderpass;
pub mod surface;
pub mod swapchain;
pub mod sync;

#[cfg(test)]
mod headless;

pub trait Wrap<T> {
    fn raw(&self) -> T;
}
