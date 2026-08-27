pub trait DataBuffer {
    fn write(&mut self, bytes: &[u8]);
    fn pad(&mut self, len: usize);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn as_slice(&self) -> &[u8];
}

impl DataBuffer for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn pad(&mut self, len: usize) {
        self.resize(self.len() + len, 0);
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn as_slice(&self) -> &[u8] {
        Vec::as_slice(self)
    }
}

pub struct FixedBuffer<const S: usize> {
    buffer: [u8; S],
    pos: usize,
}

impl<const S: usize> Default for FixedBuffer<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const S: usize> FixedBuffer<S> {
    pub fn new() -> Self {
        Self {
            buffer: [0; S],
            pos: 0,
        }
    }
}

impl<const S: usize> DataBuffer for FixedBuffer<S> {
    fn write(&mut self, bytes: &[u8]) {
        let new_pos = self.pos + bytes.len();
        debug_assert!(new_pos <= S);

        self.buffer[self.pos..new_pos].copy_from_slice(bytes);
        self.pos = new_pos
    }

    fn pad(&mut self, len: usize) {
        let new_pos = self.pos + len;
        debug_assert!(new_pos <= S);

        for b in &mut self.buffer[self.pos..new_pos] {
            *b = 0;
        }
        self.pos = new_pos
    }

    fn len(&self) -> usize {
        self.pos
    }

    fn is_empty(&self) -> bool {
        self.pos == 0
    }

    fn clear(&mut self) {
        self.pos = 0;
    }

    fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.pos]
    }
}

pub struct SliceBuffer<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

impl<'a> SliceBuffer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, pos: 0 }
    }
}

impl<'a> DataBuffer for SliceBuffer<'a> {
    fn write(&mut self, bytes: &[u8]) {
        let new_pos = self.pos + bytes.len();
        debug_assert!(new_pos <= self.buffer.len());

        self.buffer[self.pos..new_pos].copy_from_slice(bytes);
        self.pos = new_pos
    }

    fn pad(&mut self, len: usize) {
        let new_pos = self.pos + len;
        debug_assert!(new_pos <= self.buffer.len());

        for b in &mut self.buffer[self.pos..new_pos] {
            *b = 0;
        }
        self.pos = new_pos
    }

    fn len(&self) -> usize {
        self.pos
    }

    fn is_empty(&self) -> bool {
        self.pos == 0
    }

    fn clear(&mut self) {
        self.pos = 0;
    }

    fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.pos]
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

    use crate::data::data_buffer::{DataBuffer, FixedBuffer};

    fn setup() {
        let sub = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap_or_default();
    }

    #[test]
    fn test_fixed() {
        setup();

        let mut buf = FixedBuffer::<128>::new();

        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());

        buf.write(&[1, 2, 3]);
        assert_eq!(buf.len(), 3);
        assert!(!buf.is_empty());

        buf.write(&[3]);
        assert_eq!(buf.len(), 4);
        assert!(!buf.is_empty());

        buf.clear();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());

        buf.pad(10);
        assert_eq!(buf.len(), 10);
        assert!(!buf.is_empty());

        buf.write(&[3]);
        assert_eq!(buf.len(), 11);
        assert!(!buf.is_empty());

        buf.pad(10);
        assert_eq!(buf.len(), 21);
        assert!(!buf.is_empty());

        buf.clear();
        buf.pad(128);
        assert_eq!(buf.len(), 128);
        assert!(!buf.is_empty());
    }
}
