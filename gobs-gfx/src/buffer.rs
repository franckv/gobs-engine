use std::sync::Arc;

use bytemuck::Pod;
use uuid::Uuid;

use crate::Renderer;

pub type BufferId = Uuid;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BufferType {
    Vertex,
    Index,
    Staging,
    StagingDst,
    Uniform,
}

pub trait Buffer<R: Renderer> {
    fn id(&self) -> BufferId;
    fn new(name: &str, size: usize, ty: BufferType, device: &R::Device) -> Self;
    fn resize(&mut self, size: usize, device: &R::Device);
    fn copy<T: Copy>(&mut self, entries: &[T], offset: usize);
    fn size(&self) -> usize;
    fn ty(&self) -> BufferType;
    fn address(&self, device: &R::Device) -> u64;
    fn get_bytes<T: Pod>(&self, data: &mut Vec<T>);
}

pub struct BufferView<Buffer> {
    pub buffer: Arc<Buffer>,
    pub offset: u64,
    pub len: usize,
    pub count: usize,
}

impl<Buffer> Clone for BufferView<Buffer> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            offset: self.offset,
            len: self.len,
            count: self.count,
        }
    }
}
