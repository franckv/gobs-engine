#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    InvalidInitData,
    InvalidData,
    RequestTooLarge,
}

pub trait Allocation {
    fn start(&self) -> usize;
    fn size(&self) -> usize;
    fn requested_size(&self) -> usize;
}

pub trait Allocator {
    type Allocation: Allocation;

    fn allocate(&mut self, size: usize) -> Result<Self::Allocation, AllocError>;
    fn release(&mut self, allocation: Self::Allocation);
}
