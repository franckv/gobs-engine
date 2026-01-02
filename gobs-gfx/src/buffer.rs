// pub trait Buffer<R: Renderer> {
//     fn id(&self) -> BufferId;
//     fn new(name: &str, size: usize, ty: BufferType, device: &R::Device) -> Self;
//     fn resize(&mut self, size: usize, device: &R::Device);
//     fn copy<T: Copy>(&mut self, entries: &[T], offset: usize);
//     fn size(&self) -> usize;
//     fn ty(&self) -> BufferType;
//     fn address(&self, device: &R::Device) -> u64;
//     fn get_bytes<T: Pod>(&self, data: &mut Vec<T>);
// }
