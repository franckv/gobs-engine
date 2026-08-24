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

    pub fn capacity(&self) -> usize {
        self.capacity
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
    fn test_index_pool() {
        setup();

        let mut pool = IndexPool::new(4);

        let idx1 = pool.allocate();
        assert!(idx1.is_ok());
        let idx1 = idx1.unwrap();
        assert_eq!(idx1, 0);

        let idx2 = pool.allocate();
        assert!(idx2.is_ok());
        let idx2 = idx2.unwrap();
        assert_eq!(idx2, 1);

        pool.release(idx2);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pool.release(idx2)));
        assert!(result.is_err());

        let idx3 = pool.allocate();
        assert!(idx3.is_ok());
        let idx3 = idx3.unwrap();
        assert_eq!(idx3, 1);

        let idx4 = pool.allocate();
        assert!(idx4.is_ok());
        let idx4 = idx4.unwrap();
        assert_eq!(idx4, 2);

        let idx5 = pool.allocate();
        assert!(idx5.is_ok());
        let idx5 = idx5.unwrap();
        assert_eq!(idx5, 3);

        let result = pool.allocate();
        assert!(result.is_err());
    }
}
