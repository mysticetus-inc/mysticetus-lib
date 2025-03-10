use bytes::{Buf, Bytes, BytesMut};

/// Buffer type that can give out a fully initialized slice to write to,
/// unlike BytesMut itself
pub(crate) struct Buffer {
    bytes: BytesMut,
    len: usize,
}

impl Buffer {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: BytesMut::with_capacity(capacity),
            len: 0,
        }
    }

    pub(crate) fn inc_len(&mut self, inc: usize) {
        self.len += inc;
    }

    pub(crate) fn chunk_mut(&mut self) -> &mut [u8] {
        &mut self.bytes[self.len..]
    }

    pub(crate) fn grow(&mut self) {
        let grow_to = self
            .len
            .checked_mul(2)
            .unwrap()
            .max(std::mem::size_of::<usize>());

        self.bytes.resize(grow_to, 0);
    }

    pub(crate) fn take(&mut self) -> Bytes {
        let ret = self.bytes.copy_to_bytes(self.len);
        self.len = 0;
        ret
    }
}
