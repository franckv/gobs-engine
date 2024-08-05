use crate::{BufferUsage, GfxDevice};

pub trait Buffer {
    fn new(name: &str, size: usize, usage: BufferUsage, device: &GfxDevice) -> Self;
    fn copy<T: Copy>(&mut self, entries: &[T], offset: usize);
    fn size(&self) -> usize;
    fn address(&self, device: &GfxDevice) -> u64;
}
