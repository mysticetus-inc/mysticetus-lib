use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct TakeOnce<T> {
    cell: UnsafeCell<MaybeUninit<T>>,
    is_taken: AtomicBool,
}

// since this is basically a simplified mutex, reuse the same bounds.
unsafe impl<T: Send> Send for TakeOnce<T> {}
unsafe impl<T: Send> Sync for TakeOnce<T> {}

impl<T> TakeOnce<T> {
    pub fn new(x: T) -> Self {
        Self {
            is_taken: AtomicBool::new(false),
            cell: UnsafeCell::new(MaybeUninit::new(x)),
        }
    }

    pub fn reset_if_empty<F>(&mut self, f: F)
    where
        F: FnOnce() -> T,
    {
        let is_taken = self.is_taken.get_mut();

        if *is_taken {
            // SAFETY:
            // replacing the pointer is safe, since MaybeUninit::new of a valid
            // T is also valid.
            //
            // Since is_taken is true, the replaced value is uninitialized, therefore
            // we don't need to free anything. We also require a '&mut self', so no
            // other thread can cause a race condition
            unsafe {
                self.cell.get().replace(MaybeUninit::new(f()));
            }

            *is_taken = false;
        }
    }

    pub fn reset(&mut self, item: T) -> Option<T> {
        let is_taken = self.is_taken.get_mut();

        let new = MaybeUninit::new(item);

        // SAFETY:
        // ptr replace: MaybeUninit::new of a valid T is a valid replacement pointer
        // assume_init: if !is_taken, we know the inner value is still valid.
        //              reset being a method that takes a '&mut self' also ensures that
        //              no other thread can cause a race condition.
        let old = unsafe {
            let old = self.cell.get().replace(new);
            if *is_taken {
                None
            } else {
                Some(old.assume_init())
            }
        };

        *is_taken = false;
        old
    }

    pub fn has_been_taken(&self) -> bool {
        self.is_taken.load(Ordering::SeqCst)
    }

    pub fn take_checked(&self) -> Option<T> {
        let first_time = self
            .is_taken
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok();

        if !first_time {
            return None;
        }

        // SAFETY: If we were able to swap the 'is_taken' flag, we're safe swap the inner value with
        // an uninitialized variant. Since TakeOnce can only be initialized with an
        // initialized value, it'll be properly initialized.
        let value = unsafe { ptr::replace(self.cell.get(), MaybeUninit::uninit()).assume_init() };

        Some(value)
    }

    pub fn take(&self) -> T {
        match self.take_checked() {
            Some(item) => item,
            None => panic!("TakeOnce value already taken"),
        }
    }
}

impl<T> Drop for TakeOnce<T> {
    fn drop(&mut self) {
        // try and take the inner thing to let its dtor run
        let _ = self.take_checked();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};
    use std::thread;

    use super::TakeOnce;

    #[test]
    fn test_basic() {
        let once = TakeOnce::new(1);

        assert_eq!(1, once.take());
        assert!(once.take_checked().is_none());
    }

    #[test]
    fn test_threaded() {
        const THREADS: usize = 5;
        let once = Arc::new(TakeOnce::new(1));
        let barrier = Arc::new(Barrier::new(THREADS));

        let handles = (0..THREADS)
            .map(|_| {
                let clone = Arc::clone(&once);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    clone.take_checked()
                })
            })
            .collect::<Vec<_>>();

        let mut results = Vec::new();
        for handle in handles {
            if let Some(int) = handle.join().unwrap() {
                results.push(int);
            }
        }

        assert_eq!(1, results.len());
        assert_eq!(1, results[0]);
        assert!(once.take_checked().is_none());
    }
}
