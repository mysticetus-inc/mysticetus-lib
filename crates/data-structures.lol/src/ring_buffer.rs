/// A small, stack based ring buffer. TODO: Finish + write tests.
use std::mem::{self, MaybeUninit};

pub struct RingBuffer<const CAP: u16, T>
where
    [(); CAP as usize]:,
{
    buf: [MaybeUninit<T>; CAP as usize],
    /// Points at the final initialized element in 'buf', IF len != 0.
    /// if len == 1, then the head and tail should be equal.
    tail: usize,
    head: usize,
    len: usize,
}

impl<const CAP: u16, T> RingBuffer<CAP, T>
where
    [(); CAP as usize]:,
{
    pub const CAPACITY: usize = CAP as usize;

    pub const fn new() -> Self {
        assert!(CAP > 0, "capacity must be non-zero");
        Self {
            buf: MaybeUninit::uninit_array(),
            tail: 0,
            head: 0,
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == Self::CAPACITY
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.is_empty() {
            return (&[], &[]);
        }

        let dst_tail = (self.tail + 1) % Self::CAPACITY;
        if dst_tail > self.head {
            let chunk =
                unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[self.head..dst_tail]) };

            return (chunk, &[]);
        }

        let leading = unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[self.head..]) };
        let trailing = unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[..dst_tail]) };

        (leading, trailing)
    }

    pub fn make_contiguous(&mut self) -> &mut [T] {
        if self.is_empty() {
            return &mut [];
        }

        self.buf.rotate_left(self.head);
        self.head = 0;

        let dst_tail = self.head + self.len;
        self.tail = dst_tail - 1;

        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf[self.head..dst_tail]) }
    }

    pub const fn push_back(&mut self, item: T) -> Result<(), T> {
        let next_tail = (self.tail + 1) % Self::CAPACITY;

        if next_tail == self.head {
            return Err(item);
        }

        self.buf[next_tail].write(item);

        self.len += 1;
        self.tail = next_tail;

        Ok(())
    }

    pub const fn push_front(&mut self, item: T) -> Result<(), T> {
        let next_head = (self.head + Self::CAPACITY - 1) % Self::CAPACITY;

        if next_head == self.tail {
            return Err(item);
        }

        self.buf[next_head].write(item);

        self.len += 1;
        self.head = next_head;

        Ok(())
    }

    pub const fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let popped = mem::replace(&mut self.buf[self.tail], MaybeUninit::uninit());

        self.tail = (self.tail + Self::CAPACITY - 1) % Self::CAPACITY;
        self.len -= 1;

        Some(unsafe { popped.assume_init() })
    }

    pub const fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let popped = mem::replace(&mut self.buf[self.head], MaybeUninit::uninit());

        self.head = (self.head + 1) % Self::CAPACITY;
        self.len -= 1;

        Some(unsafe { popped.assume_init() })
    }

    pub const fn peek_front(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }

        Some(unsafe { self.buf[self.head].assume_init_ref() })
    }

    pub const fn peek_back(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }

        Some(unsafe { self.buf[self.tail].assume_init_ref() })
    }

    pub const fn peek_front_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }

        Some(unsafe { self.buf[self.head].assume_init_mut() })
    }

    pub const fn peek_back_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }

        Some(unsafe { self.buf[self.tail].assume_init_mut() })
    }
}
