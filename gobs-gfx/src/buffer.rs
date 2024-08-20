use uuid::Uuid;

use crate::BufferUsage;

pub type BufferId = Uuid;

pub trait Buffer {
    type GfxDevice;

    fn id(&self) -> BufferId;
    fn new(name: &str, size: usize, usage: BufferUsage, device: &Self::GfxDevice) -> Self;
    fn copy<T: Copy>(&mut self, entries: &[T], offset: usize);
    fn size(&self) -> usize;
    fn address(&self, device: &Self::GfxDevice) -> u64;
}
