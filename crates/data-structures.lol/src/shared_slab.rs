use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam::queue::SegQueue;
use tokio::sync::oneshot;

pub struct Pool<const CAPACITY: usize, T> {
    pool: [Arc<slot::Slot<T>>; CAPACITY],
    last_initialized: AtomicUsize,
    last_borrowed: AtomicUsize,
    alive: AtomicUsize,
    waiters: Arc<SegQueue<oneshot::Sender<T>>>,
}

pub struct Borrowed<T> {
    dropped: bool,
    value: ManuallyDrop<T>,
    slot: Arc<slot::Slot<T>>,
    watiers: Arc<SegQueue<oneshot::Sender<T>>>,
    dont_return: bool,
}

impl<T> Drop for Borrowed<T> {
    fn drop(&mut self) {
        if self.dropped {
            return;
        }

        // SAFETY: we use 'dropped' to track whether or not this was dropped or is still alive
        let mut value = unsafe { ManuallyDrop::take(&mut self.value) };

        if self.dont_return {
            return;
        }

        // first, try and immediately pass it off to something that's waiting for one.
        while let Some(waiter) = self.watiers.pop() {
            match waiter.send(value) {
                Ok(_) => return,
                Err(item) => value = item,
            }
        }
    }
}

impl<const CAPACITY: usize, T> Pool<CAPACITY, T> {
    pub fn return_item(&self, mut item: T) -> Result<(), T> {
        loop {}

        let last = self.last_borrowed.load(Ordering::Relaxed);

        todo!()
    }
}

mod slot {
    use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
    use std::sync::{Mutex, PoisonError};

    const UNINITIALIZED: u8 = 0b00;
    const INITIALIZED: u8 = 0b01;
    const BORROWED: u8 = 0b11;

    pub(super) struct Slot<T> {
        state: AtomicU8,
        item: Mutex<Option<T>>,
    }

    impl<T> Slot<T> {
        pub(super) const fn new() -> Self {
            Self {
                state: AtomicU8::new(UNINITIALIZED),
                item: Mutex::new(None),
            }
        }

        fn insert_inner(&self, expected_state: u8, value: T) -> Option<T> {
            if self
                .state
                .compare_exchange(
                    expected_state,
                    INITIALIZED,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                )
                .is_err()
            {
                return Some(value);
            }

            self.item
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .replace(value)
        }

        pub(super) fn return_item(&self, value: T) -> Option<T> {
            self.insert_inner(BORROWED, value)
        }

        pub(super) fn initialize(&self, value: T) -> Option<T> {
            self.insert_inner(UNINITIALIZED, value)
        }

        pub(super) fn take(&self) -> Option<T> {
            self.state.store(UNINITIALIZED, Ordering::SeqCst);
            self.item
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take()
        }

        pub(super) fn borrow(&self) -> Option<T> {
            if self
                .state
                .compare_exchange(INITIALIZED, BORROWED, Ordering::SeqCst, Ordering::Relaxed)
                .is_err()
            {
                return None;
            }

            let item = self
                .item
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take();

            debug_assert!(item.is_some());
            // if for some reason the state gets out of sync,
            if item.is_none() {}

            item
        }
    }
}
