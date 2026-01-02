use std::sync::Arc;

use gobs_vulkan::buffers::Buffer;

pub struct BufferView {
    pub buffer: Buffer,
    pub offset: u64,
    pub len: usize,
}
