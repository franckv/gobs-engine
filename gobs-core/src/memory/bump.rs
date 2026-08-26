use crate::memory::allocator::{AllocError, Allocation, Allocator};

#[derive(Clone)]
pub struct BumpAllocation {
    start: usize,
    size: usize,
    requested_size: usize,
}

impl Allocation for BumpAllocation {
    fn start(&self) -> usize {
        self.start
    }

    fn size(&self) -> usize {
        self.size
    }

    fn requested_size(&self) -> usize {
        self.requested_size
    }
}

impl BumpAllocation {
    pub fn new(start: usize, size: usize, requested_size: usize) -> Self {
        Self {
            start,
            size,
            requested_size,
        }
    }
}

pub struct BumpAllocator {
    cursor: usize,
    capacity: usize,
}

impl Allocator for BumpAllocator {
    type Allocation = BumpAllocation;

    fn allocate(&mut self, size: usize) -> Result<Self::Allocation, AllocError> {
        if self.cursor + size > self.capacity {
            return Err(AllocError::RequestTooLarge);
        }

        let start = self.cursor;
        self.cursor += size;

        Ok(BumpAllocation::new(start, size, size))
    }

    fn release(&mut self, _allocation: Self::Allocation) {
        unimplemented!()
    }

    fn clear(&mut self) {
        self.cursor = 0;
    }
}

impl BumpAllocator {
    pub fn new(capacity: usize) -> Self {
        Self {
            cursor: 0,
            capacity,
        }
    }
}

#[cfg(test)]
mod test {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use super::*;

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_bump() {
        setup();

        let mut allocator = BumpAllocator::new(16);

        let alloc1 = allocator.allocate(4);
        assert!(alloc1.is_ok());
        let alloc1 = alloc1.unwrap();
        assert_eq!(alloc1.size(), 4);
        assert_eq!(alloc1.requested_size(), 4);
        assert_eq!(alloc1.start(), 0);

        let alloc2 = allocator.allocate(10);
        assert!(alloc2.is_ok());
        let alloc2 = alloc2.unwrap();
        assert_eq!(alloc2.size(), 10);
        assert_eq!(alloc2.requested_size(), 10);
        assert_eq!(alloc2.start(), 4);

        let result = allocator.allocate(3);
        assert!(result.is_err());

        let alloc3 = allocator.allocate(2);
        assert!(alloc3.is_ok());
        let alloc3 = alloc3.unwrap();
        assert_eq!(alloc3.size(), 2);
        assert_eq!(alloc3.requested_size(), 2);
        assert_eq!(alloc3.start(), 14);

        allocator.clear();

        let alloc4 = allocator.allocate(4);
        assert!(alloc4.is_ok());
        let alloc4 = alloc4.unwrap();
        assert_eq!(alloc4.size(), 4);
        assert_eq!(alloc4.requested_size(), 4);
        assert_eq!(alloc4.start(), 0);
    }
}
