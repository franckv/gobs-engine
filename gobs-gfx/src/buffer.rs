use uuid::Uuid;

use crate::BufferUsage;
use crate::Renderer;

pub type BufferId = Uuid;

pub trait Buffer<R: Renderer> {
    fn id(&self) -> BufferId;
    fn new(name: &str, size: usize, usage: BufferUsage, device: &R::Device) -> Self;
    fn copy<T: Copy>(&mut self, entries: &[T], offset: usize);
    fn size(&self) -> usize;
    fn address(&self, device: &R::Device) -> u64;
}
