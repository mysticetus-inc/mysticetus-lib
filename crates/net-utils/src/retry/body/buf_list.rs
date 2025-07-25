use std::collections::VecDeque;

use bytes::{Buf, BufMut, Bytes};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BufList {
    chunks: VecDeque<Bytes>,
    remaining: usize,
}

impl BufList {
    pub fn push(&mut self, bytes: Bytes) {
        if !bytes.is_empty() {
            self.remaining += bytes.len();
            self.chunks.push_back(bytes);
        }
    }

    pub fn pop(&mut self) -> Option<Bytes> {
        let front = self.chunks.pop_front()?;
        self.remaining -= front.len();
        Some(front)
    }
}

impl Buf for BufList {
    #[inline]
    fn chunk(&self) -> &[u8] {
        self.chunks.front().map(Buf::chunk).unwrap_or_default()
    }

    #[inline]
    fn advance(&mut self, mut cnt: usize) {
        while 0 < cnt {
            let front = self
                .chunks
                .front_mut()
                .expect("advance called while BufList is empty");
            let to_advance = std::cmp::min(cnt, front.len());
            front.advance(to_advance);
            self.remaining -= to_advance;
            cnt -= to_advance;
        }
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.remaining
    }

    #[inline]
    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        match self.chunks.front_mut() {
            // optimized case that avoids copying
            Some(front) if len <= front.len() => {
                self.remaining -= len;
                front.copy_to_bytes(len)
            }
            _ if len == 0 => bytes::Bytes::new(),
            _ => {
                let mut dst = bytes::BytesMut::with_capacity(len);
                dst.put(self.take(len));
                dst.freeze()
            }
        }
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [std::io::IoSlice<'a>]) -> usize {
        self.chunks
            .iter()
            .zip(dst)
            .map(|(chunk, dst)| *dst = std::io::IoSlice::new(chunk))
            .count()
    }
}
