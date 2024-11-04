use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, PoisonError};

pub trait Buffer: Sized + Default + Clone {
    fn capacity(&self) -> usize;

    fn clear(&mut self);

    fn reserve(&mut self, additional: usize);

    fn with_capacity(capacity: usize) -> Self;
}

#[derive(Debug, Clone)]
pub struct ReusableBufs<B: Buffer> {
    bufs: Arc<Mutex<inner::Inner<B>>>,
}

impl<B: Buffer> ReusableBufs<B> {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            bufs: Arc::new(Mutex::new(inner::Inner::with_capacity(cap))),
        }
    }

    pub fn take_reusable(&self, desired_cap: Option<usize>) -> Reusable<B> {
        let buf = self.take_buf(desired_cap);

        Reusable {
            buf,
            parent: self.clone(),
        }
    }

    pub fn take_buf(&self, desired_cap: Option<usize>) -> B {
        let popped = self
            .bufs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop();

        match popped {
            Some(mut buf) => {
                buf.clear();
                if let Some(cap) = desired_cap {
                    buf.reserve(cap);
                }
                buf
            }
            None => match desired_cap {
                None => B::default(),
                Some(cap) => B::with_capacity(cap),
            },
        }
    }

    pub fn return_buf(&self, buf: B) {
        self.bufs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(buf);
    }
}

pub struct Reusable<B: Buffer> {
    buf: B,
    parent: ReusableBufs<B>,
}

impl<B: Buffer> Deref for Reusable<B> {
    type Target = B;
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<B: Buffer> DerefMut for Reusable<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl<B: Buffer> Drop for Reusable<B> {
    fn drop(&mut self) {
        if self.buf.capacity() > 0 {
            self.parent.return_buf(std::mem::take(&mut self.buf))
        }
    }
}

mod inner {
    use std::collections::BinaryHeap;

    use super::Buffer;

    #[derive(Debug, Clone)]
    pub(super) struct Inner<B: Buffer> {
        buf: BinaryHeap<CapacityCmp<B>>,
    }

    impl<B: Buffer> Inner<B> {
        pub(super) fn with_capacity(capacity: usize) -> Self {
            Self {
                buf: BinaryHeap::with_capacity(capacity),
            }
        }

        pub(super) fn insert(&mut self, buf: B) {
            self.buf.push(CapacityCmp(buf));
        }

        pub(super) fn pop(&mut self) -> Option<B> {
            self.buf.pop().map(|wrapped| wrapped.0)
        }
    }

    #[derive(Debug, Clone)]
    #[repr(transparent)]
    struct CapacityCmp<B>(B);

    impl<B: Buffer> PartialEq for CapacityCmp<B> {
        fn eq(&self, other: &Self) -> bool {
            self.0.capacity() == other.0.capacity()
        }
    }

    impl<B: Buffer> Eq for CapacityCmp<B> {}

    impl<B: Buffer> PartialOrd for CapacityCmp<B> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<B: Buffer> Ord for CapacityCmp<B> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.capacity().cmp(&other.0.capacity())
        }
    }
}

macro_rules! impl_buffer {
    ($($t:ty),* $(,)?) => {
        $(
            impl Buffer for $t {
                #[inline]
                fn capacity(&self) -> usize {
                    Self::capacity(self)
                }

                #[inline]
                fn clear(&mut self) {
                    Self::clear(self)
                }

                #[inline]
                fn reserve(&mut self, additional: usize) {
                    Self::reserve(self, additional)
                }

                #[inline]
                fn with_capacity(capacity: usize) -> Self {
                    Self::with_capacity(capacity)
                }
            }
        )*
    };
}

impl_buffer!(String, Vec<u8>);
