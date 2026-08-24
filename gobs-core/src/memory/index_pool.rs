use crate::memory::allocator::AllocError;

pub struct IndexPool {
    free_list: Vec<usize>,
    capacity: usize,
    border: usize,
}

impl IndexPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            free_list: Vec::new(),
            capacity,
            border: 0,
        }
    }

    pub fn allocate(&mut self) -> Result<usize, AllocError> {
        if let Some(index) = self.free_list.pop() {
            Ok(index)
        } else {
            let index = self.border;
            self.border += 1;

            if self.border > self.capacity {
                Err(AllocError::OutOfMemory)
            } else {
                Ok(index)
            }
        }
    }

    pub fn release(&mut self, index: usize) {
        debug_assert!(!self.free_list.contains(&index));

        self.free_list.push(index);
    }
}
