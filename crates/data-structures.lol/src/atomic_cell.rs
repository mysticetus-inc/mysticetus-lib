use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

/// A atomic-based type that acts similar to [`Cell<Option<T>>`], but is also [`Send`] + [`Sync`].
///
/// [`Cell<Option<T>>`]: std::cell::Cell<Option<T>>
pub struct AtomicCell<T> {
    // the inner pointer, where null represents an empty cell.
    ptr: AtomicPtr<T>,
}

impl<T> AtomicCell<T> {
    pub const fn empty() -> Self {
        Self {
            ptr: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn new_from_box(item: Box<T>) -> Self {
        Self {
            ptr: AtomicPtr::new(Box::leak(item) as *mut T),
        }
    }

    pub fn get_mut_ptr(&mut self) -> &mut *mut T {
        self.ptr.get_mut()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        let ptr = self.ptr.get_mut();

        if ptr.is_null() {
            None
        } else {
            // SAFETY: if ptr is not null, this is a valid T (allocated by Box).
            // We can return a '&mut T', since this method itself requires '&mut self'
            Some(unsafe { &mut **ptr })
        }
    }

    pub fn new(item: T) -> Self {
        Self::new_from_box(Box::new(item))
    }

    pub fn set_with<F>(&self, f: F) -> Option<Box<T>>
    where
        F: FnOnce() -> T,
    {
        let old = self.ptr.load(Ordering::Acquire);

        let new = Box::leak(Box::new(f())) as *mut T;

        self.ptr.store(new, Ordering::Release);

        if old.is_null() {
            None
        } else {
            // SAFETY: Since 'prev' is not null, this pointer was previously
            // created from a [`Box::leak`] call, making this safe.
            unsafe { Some(Box::from_raw(old)) }
        }
    }

    fn swap_inner(&self, dst: *mut T, ordering: Ordering) -> Option<Box<T>> {
        let prev = self.ptr.swap(dst, ordering);

        if prev.is_null() {
            return None;
        }

        // SAFETY: Since 'prev' is not null, this pointer was previously
        // created from a [`Box::leak`] call, making this safe.
        unsafe { Some(Box::from_raw(prev)) }
    }

    pub fn is_some(&self, ordering: Ordering) -> bool {
        !self.ptr.load(ordering).is_null()
    }

    pub fn is_none(&self, ordering: Ordering) -> bool {
        !self.is_some(ordering)
    }

    pub fn set_from_box(&self, item: Box<T>, ordering: Ordering) -> Option<Box<T>> {
        self.swap_inner(Box::leak(item), ordering)
    }

    pub fn set(&self, item: T, ordering: Ordering) -> Option<T> {
        self.swap_inner(Box::leak(Box::new(item)), ordering)
            .map(Box::into_inner)
    }

    pub fn take_boxed(&self, ordering: Ordering) -> Option<Box<T>> {
        let prev = self.ptr.swap(ptr::null_mut(), ordering);
        if prev.is_null() {
            return None;
        }

        // SAFETY: Since 'prev' is not null, this pointer was previously
        // created from a [`Box::leak`] call, making this safe.
        unsafe { Some(Box::from_raw(prev)) }
    }

    pub fn take(&self, ordering: Ordering) -> Option<T> {
        self.take_boxed(ordering).map(Box::into_inner)
    }
}

impl<T> Drop for AtomicCell<T> {
    fn drop(&mut self) {
        // remove any inner value, letting the Box dtor do the heavy lifting.
        let _ = self.take_boxed(Ordering::SeqCst);
    }
}

#[test]
fn assert_send_sync() {
    fn inner<T: Send + Sync>() {}

    inner::<AtomicCell<*const ()>>()
}

#[cfg(loom)]
#[test]
fn loom_test_atomic_cell() {
    use std::sync::Barrier;

    use loom::sync::Arc;

    use crate::{hint, thread};

    // the maximum number of allowed threads, minus the current/main thread
    const THREADS: usize = loom::MAX_THREADS - 2;

    loom::model(|| {
        let shared = Arc::new(AtomicCell::empty());
        let mut handles = Vec::with_capacity(THREADS);

        for _ in 0..THREADS {
            let shared_clone = Arc::clone(&shared);

            handles.push(thread::spawn(move || {
                shared_clone.set_with(std::time::Instant::now)
            }));
        }

        let mut handle_results = Vec::with_capacity(THREADS);

        for handle in handles {
            if let Some(instant) = handle.join().unwrap() {
                handle_results.push(*instant);
            }
        }

        let max_instant = handle_results.into_iter().max().unwrap();

        let last_set_instant = shared.take(Ordering::SeqCst).unwrap();

        assert!(last_set_instant > max_instant);
    });
}
