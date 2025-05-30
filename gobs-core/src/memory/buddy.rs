use thiserror::Error;

#[derive(Debug, Error)]
pub enum AllocError {
    #[error("out of memory")]
    OutOfMemory,
    #[error("invalid block index")]
    InvalidBlockIndex,
    #[error("order too high")]
    InvalidOrder,
}

pub struct Allocation {
    order: usize,
    idx: usize,
    pub memory_start: usize,
}

impl Allocation {
    fn new(order: usize, idx: usize, memory_start: usize) -> Self {
        Self {
            order,
            idx,
            memory_start,
        }
    }
}

pub struct BuddyAllocator {
    order: usize,
    memory_start: usize,
    memory_size: usize,
    block_size: usize,
    free_blocks: Vec<u128>,
}

impl BuddyAllocator {
    pub fn new(size: usize, order: usize) -> Result<Self, AllocError> {
        let log2 = size.ilog2();
        let max_size = 2_usize.pow(log2);
        let free_blocks = (0..=order).map(|i| if i == 0 { 1 } else { 0 }).collect();

        let block_count = 2_usize.pow(order as u32);

        if block_count > 128 {
            return Err(AllocError::InvalidOrder);
        }

        Ok(Self {
            order,
            memory_start: 0,
            memory_size: max_size,
            block_size: max_size >> order,
            free_blocks,
        })
    }

    fn new_allocation(&self, order: usize, idx: usize) -> Allocation {
        Allocation::new(order, idx, self.block_start(order, idx))
    }

    fn block_size(&self, order: usize) -> usize {
        self.memory_size >> order
    }

    fn block_start(&self, order: usize, idx: usize) -> usize {
        self.memory_start + (self.block_count(order) - idx - 1) * self.block_size(order)
    }

    fn has_free(&self, order: usize) -> bool {
        self.free_blocks[order] != 0
    }

    fn is_free(&self, order: usize, idx: usize) -> bool {
        self.free_blocks[order] & 1 << idx != 0
    }

    fn buddy_idx(&self, idx: usize) -> usize {
        idx ^ 1
    }

    fn parent_idx(&self, idx: usize) -> usize {
        idx / 2
    }

    fn toggle_free_block(&mut self, order: usize, idx: usize) {
        self.free_blocks[order] ^= 1 << idx;
    }

    fn split_block(&mut self, order: usize, idx: usize) {
        tracing::trace!(target: "alloc", "splitting block {},{}", order, idx);

        self.toggle_free_block(order, idx);
        self.toggle_free_block(order + 1, 2 * idx);
        self.toggle_free_block(order + 1, 2 * idx + 1);
    }

    fn merge_block(&mut self, order: usize, idx: usize) {
        tracing::trace!(target: "alloc", "merging block {},{}", order, idx);

        assert!(
            self.is_free(order, idx)
                && self.is_free(order, self.buddy_idx(idx))
                && !self.is_free(order - 1, self.parent_idx(idx))
        );

        self.toggle_free_block(order, idx);
        self.toggle_free_block(order, self.buddy_idx(idx));
        self.toggle_free_block(order - 1, self.parent_idx(idx));
    }

    fn block_count(&self, order: usize) -> usize {
        2_usize.pow(order as u32)
    }

    fn get_free_index(&self, order: usize) -> Result<usize, AllocError> {
        for i in 0..self.block_count(order) {
            if self.is_free(order, i) {
                return Ok(i);
            }
        }

        Err(AllocError::InvalidBlockIndex)
    }

    fn get_order(&self, size: usize) -> Result<usize, AllocError> {
        for i in (0..=self.order).rev() {
            if self.block_size(i) >= size {
                return Ok(i);
            }
        }

        Err(AllocError::OutOfMemory)
    }

    fn block_format(&self, order: usize) -> String {
        let min_block_format = format!("[{}]", self.block_size);

        if order == self.order {
            min_block_format.to_string()
        } else {
            let min_block_len = min_block_format.len();
            let unpadded_format = format!("[{}]", self.block_size(order));
            let block_len = min_block_len * 2_usize.pow(self.order as u32 - order as u32);

            let padding = block_len - unpadded_format.len();
            let left_padding = padding / 2;
            let right_padding = padding - left_padding;

            let block_format = format!(
                "[{:left$}{}{:right$}]",
                "",
                self.block_size(order),
                "",
                left = left_padding,
                right = right_padding,
            );

            block_format.to_string()
        }
    }

    pub fn memory_format(&self) -> String {
        let label_len = format!("{}", self.block_size).len();
        let label = format!("[{}]", (0..label_len).map(|_| 'x').collect::<String>());
        let mut block = (0..self.block_count(self.order))
            .map(|_| label.to_string())
            .collect::<String>();

        for i in 0..=self.order {
            if self.has_free(i) {
                let n_blocks = self.block_count(i);
                for j in 0..n_blocks {
                    if self.is_free(i, j) {
                        let sub_block = self.block_format(i);
                        let block_pos = (n_blocks - j - 1) * sub_block.len();
                        block.replace_range(block_pos..(block_pos + sub_block.len()), &sub_block);
                    }
                }
            }
        }

        block
    }

    pub fn allocate(&mut self, size: usize) -> Result<Allocation, AllocError> {
        let target_order = self.get_order(size)?;

        tracing::trace!(target: "alloc", "allocate {}, target order {}", size, target_order);

        if !self.has_free(target_order) {
            let mut start_order = target_order;
            // find first free parent node
            for i in (0..target_order).rev() {
                if self.has_free(i) {
                    start_order = i;
                    break;
                }
            }
            tracing::trace!(target: "alloc", "starting from {}", start_order);

            // split blocks from parent to target
            if start_order < target_order {
                for i in start_order..target_order {
                    let idx = self.get_free_index(i)?;
                    self.split_block(i, idx);
                }
            }
        }

        if self.has_free(target_order) {
            let idx = self.get_free_index(target_order)?;
            self.toggle_free_block(target_order, idx);
            return Ok(self.new_allocation(target_order, idx));
        }

        Err(AllocError::OutOfMemory)
    }

    pub fn release(&mut self, allocation: Allocation) {
        tracing::trace!(target: "alloc", "release block {},{}", allocation.order, allocation.idx);

        self.toggle_free_block(allocation.order, allocation.idx);

        let mut idx = allocation.idx;
        for i in (1..=allocation.order).rev() {
            if self.is_free(i, self.buddy_idx(idx)) {
                self.merge_block(i, idx);
                idx = self.parent_idx(idx);
            } else {
                break;
            }
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
    fn test_allocator() {
        setup();

        let mut buddy = BuddyAllocator::new(1024, 3).unwrap();

        assert!(buddy.has_free(0));
        assert!(!buddy.has_free(1));
        assert_eq!(buddy.memory_size, 1024);
        assert_eq!(buddy.block_size, 128);
        assert_eq!(buddy.block_start(3, 0), buddy.memory_start + 7 * 128);
        assert_eq!(buddy.block_start(3, 5), buddy.memory_start + 2 * 128);
        assert_eq!(buddy.block_start(3, 7), buddy.memory_start);

        println!("{}", buddy.block_format(0));
        println!("{block}{block}", block = buddy.block_format(1));
        println!(
            "{block}{block}{block}{block}",
            block = buddy.block_format(2)
        );
        println!(
            "{block}{block}{block}{block}{block}{block}{block}{block}",
            block = buddy.block_format(3)
        );

        println!("Blocks:\n======\n{}", buddy.memory_format());

        let alloc1 = buddy.allocate(128);
        println!("Blocks:\n======\n{}", buddy.memory_format());
        assert!(alloc1.is_ok());

        let alloc2 = buddy.allocate(256);
        println!("Blocks:\n======\n{}", buddy.memory_format());
        assert!(alloc2.is_ok());

        let alloc3 = buddy.allocate(128);
        println!("Blocks:\n======\n{}", buddy.memory_format());
        assert!(alloc3.is_ok());

        let alloc4 = buddy.allocate(128);
        println!("Blocks:\n======\n{}", buddy.memory_format());
        assert!(alloc4.is_ok());

        let err = buddy.allocate(512);
        assert!(err.is_err());

        buddy.release(alloc4.unwrap());
        println!("Blocks:\n======\n{}", buddy.memory_format());

        buddy.release(alloc3.unwrap());
        println!("Blocks:\n======\n{}", buddy.memory_format());

        buddy.release(alloc1.unwrap());
        println!("Blocks:\n======\n{}", buddy.memory_format());

        buddy.release(alloc2.unwrap());
        println!("Blocks:\n======\n{}", buddy.memory_format());
    }
}
