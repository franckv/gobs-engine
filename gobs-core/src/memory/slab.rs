use crate::memory::{
    allocator::{AllocError, Allocation, Allocator},
    index_pool::IndexPool,
};

#[derive(Clone)]
pub struct SlabAllocation {
    idx: usize,
    size: usize,
    requested_size: usize,
}

impl Allocation for SlabAllocation {
    fn start(&self) -> usize {
        self.idx * self.size
    }

    fn size(&self) -> usize {
        self.size
    }

    fn requested_size(&self) -> usize {
        self.requested_size
    }
}

impl SlabAllocation {
    pub fn new(idx: usize, size: usize, requested_size: usize) -> Self {
        Self {
            idx,
            size,
            requested_size,
        }
    }
}

pub struct SlabAllocator {
    slot_size: usize,
    index_pool: IndexPool,
}

impl Allocator for SlabAllocator {
    type Allocation = SlabAllocation;

    fn allocate(&mut self, size: usize) -> Result<Self::Allocation, AllocError> {
        if size > self.slot_size {
            return Err(AllocError::RequestTooLarge);
        }

        let slot = self.index_pool.allocate()?;

        Ok(SlabAllocation::new(slot, self.slot_size, size))
    }

    fn release(&mut self, allocation: Self::Allocation) {
        self.index_pool.release(allocation.idx);
    }

    fn clear(&mut self) {
        self.index_pool.clear();
    }
}

impl SlabAllocator {
    pub fn new(slot_size: usize, capacity: usize) -> Self {
        let index_pool = IndexPool::new(capacity);

        Self {
            slot_size,
            index_pool,
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
    fn test_slab() {
        setup();

        let mut allocator = SlabAllocator::new(16, 4);

        let alloc1 = allocator.allocate(16);
        assert!(alloc1.is_ok());
        let alloc1 = alloc1.unwrap();
        assert_eq!(alloc1.size(), 16);
        assert_eq!(alloc1.requested_size(), 16);
        assert_eq!(alloc1.start(), 0);

        let alloc2 = allocator.allocate(12);
        assert!(alloc2.is_ok());
        let alloc2 = alloc2.unwrap();
        assert_eq!(alloc2.size(), 16);
        assert_eq!(alloc2.requested_size(), 12);
        assert_eq!(alloc2.start(), 16);

        let alloc3 = allocator.allocate(12);
        assert!(alloc3.is_ok());
        let alloc3 = alloc3.unwrap();
        assert_eq!(alloc3.size(), 16);
        assert_eq!(alloc3.requested_size(), 12);
        assert_eq!(alloc3.start(), 32);

        allocator.release(alloc2.clone());

        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| allocator.release(alloc2)));
        assert!(result.is_err());

        let alloc4 = allocator.allocate(12);
        assert!(alloc4.is_ok());
        let alloc4 = alloc4.unwrap();
        assert_eq!(alloc4.size(), 16);
        assert_eq!(alloc4.requested_size(), 12);
        assert_eq!(alloc4.start(), 16);

        let result = allocator.allocate(17);
        assert!(result.is_err());

        let alloc5 = allocator.allocate(12);
        assert!(alloc5.is_ok());
        let alloc5 = alloc5.unwrap();
        assert_eq!(alloc5.size(), 16);
        assert_eq!(alloc5.requested_size(), 12);
        assert_eq!(alloc5.start(), 48);

        let result = allocator.allocate(16);
        assert!(result.is_err());

        allocator.clear();

        let alloc6 = allocator.allocate(16);
        assert!(alloc6.is_ok());
        let alloc6 = alloc6.unwrap();
        assert_eq!(alloc6.size(), 16);
        assert_eq!(alloc6.requested_size(), 16);
        assert_eq!(alloc6.start(), 0);
    }
}
