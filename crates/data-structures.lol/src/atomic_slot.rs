//! A slot in memory that can have values atomically inserted and removed.
//!
//! Think of this as a combination of an [`Option`] and [`Cell`] with atomic operations, but
//! without a strict [`Copy`] restriction (internally uses [`Clone`]). This is handled by
//! allowing a value to *only* be inserted, or removed, with no way to obtain a reference to the
//! inner object, shared or mutable.
//!
//! [`Cell`]: std::cell::Cell

use crate::cell::UnsafeCell;
use crate::sync::atomic::{AtomicBool, Ordering};
use crate::{hint, thread};

#[derive(Debug)]
pub struct AtomicSlot<T> {
    flag: AtomicBool,
    slot: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send> Send for AtomicSlot<T> {}
unsafe impl<T: Send> Sync for AtomicSlot<T> {}

impl<T> AtomicSlot<T> {
    pub fn empty() -> Self {
        Self {
            flag: AtomicBool::new(false),
            slot: UnsafeCell::new(None),
        }
    }

    pub fn new(value: T) -> Self {
        Self {
            flag: AtomicBool::new(false),
            slot: UnsafeCell::new(Some(value)),
        }
    }

    fn with<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&mut Option<T>) -> O,
    {
        while self
            .flag
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            let mut spins = 100;

            // spin loop for 100 iterations,
            while self.flag.load(Ordering::Relaxed) && spins > 0 {
                hint::spin_loop();
                spins -= 1;
            }

            // if we spun and still didnt see that the aquire was released, yield
            if spins == 0 {
                thread::yield_now();
            }
        }

        #[cfg(not(loom))]
        let ret = f(unsafe { &mut *self.slot.get() });
        #[cfg(loom)]
        let ret = unsafe { self.slot.get_mut().with(|ptr| f(&mut *ptr)) };

        self.flag.store(false, Ordering::Release);

        ret
    }

    /// Gets the current value, without removing it. Similar to [`Cell::get`].
    ///
    /// [`Cell::get`]: [`std::cell::Cell::get`]
    pub fn get(&self) -> Option<T>
    where
        T: Copy,
    {
        self.with(|opt| *opt)
    }

    /// Identical to [`AtomicSlot::get`], but uses [`Clone`] to get the inner value.
    pub fn get_clone(&self) -> Option<T>
    where
        T: Clone,
    {
        self.with(|opt| opt.clone())
    }

    pub fn take(&self) -> Option<T> {
        self.with(Option::take)
    }

    pub fn set(&self, new: T) -> Option<T> {
        self.with(|opt| std::mem::replace(opt, Some(new)))
    }

    pub fn set_with<F>(&self, f: F) -> Option<T>
    where
        F: FnOnce() -> T,
    {
        self.with(|opt| std::mem::replace(opt, Some(f())))
    }

    pub fn set_with_copy<F>(&self, f: F) -> (T, Option<T>)
    where
        F: FnOnce() -> T,
        T: Copy,
    {
        self.with(|opt| {
            let new = f();
            (new, std::mem::replace(opt, Some(new)))
        })
    }
}

#[cfg(loom)]
#[test]
fn loom_test_atomic_slot() {
    use std::sync::Barrier;

    use loom::sync::Arc;

    // the maximum number of allowed threads, minus the current/main thread
    const THREADS: usize = loom::MAX_THREADS - 2;

    loom::model(|| {
        let shared = Arc::new(AtomicSlot::empty());
        let mut handles = Vec::with_capacity(THREADS);

        for _ in 0..THREADS {
            let shared_clone = Arc::clone(&shared);

            handles.push(thread::spawn(move || {
                shared_clone.set_with_copy(std::time::Instant::now).0
            }));
        }

        let mut handle_results = Vec::with_capacity(THREADS);

        for handle in handles {
            handle_results.push(handle.join().unwrap());
        }

        let max_instant = handle_results.into_iter().max().unwrap();

        let last_set_instant = shared.take().unwrap();

        assert_eq!(last_set_instant, max_instant);
    });
}
