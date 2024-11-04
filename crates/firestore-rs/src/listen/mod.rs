use std::cell::RefCell;
use std::num::NonZeroI32;

use rand::Rng;
use rand::rngs::ThreadRng;

mod listener;
pub use listener::Listener;

/// Opaque listener ID. Must be positive and non-zero. Using [`NonZeroU32`] would make more sense
/// here, but since the proto definition needs an [`i32`], this is easier than dealing with
/// wrapping a [`u32`] to be a valid [`i32`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListenerId(NonZeroI32);

impl ListenerId {
    // SAFETY: 1 is non-zero.
    pub const MIN: Self = unsafe { Self(NonZeroI32::new_unchecked(1)) };

    pub const MAX: Self = Self(NonZeroI32::MAX);

    pub const RANGE: std::ops::Range<Self> = Self::MIN..Self::MAX;

    pub const fn next_id(self) -> Self {
        match Self::new(self.0.get().wrapping_add(1)) {
            Some(valid) => valid,
            _ => Self::MIN,
        }
    }

    pub const fn saturating_next_id(self) -> Self {
        match Self::new(self.0.get().saturating_add(1)) {
            Some(valid) => valid,
            None => Self::MAX,
        }
    }

    const fn get(self) -> i32 {
        self.0.get()
    }

    const fn from_non_zero(id: NonZeroI32) -> Option<Self> {
        if id.get().is_positive() {
            Some(Self(id))
        } else {
            None
        }
    }

    fn rand<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        // SAFETY: using min/max as bounds will make sure the value is valud.
        unsafe { Self::new_unchecked(rng.gen_range(Self::MIN.0.get()..=Self::MAX.0.get())) }
    }

    /// Shorthand for creating a thread local [`rand::rngs::ThreadRng`] and calling
    /// [`Self::rand`] with it.
    fn gen_rand() -> Self {
        thread_local!(static RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng()));

        RNG.with_borrow_mut(Self::rand)
    }

    const fn new(id: i32) -> Option<Self> {
        match NonZeroI32::new(id) {
            None => None,
            Some(nonzero) => Self::from_non_zero(nonzero),
        }
    }

    const unsafe fn new_unchecked(id: i32) -> Self {
        // SAFETY: upheld by caller
        #[allow(unused_unsafe)]
        unsafe {
            Self(NonZeroI32::new_unchecked(id))
        }
    }
}
