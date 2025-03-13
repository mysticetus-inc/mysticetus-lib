//! A universal exponential backoff mechanism.

use std::future::IntoFuture;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use rand::Rng;
use timestamp::Duration;
use tokio::time::Sleep;

/// Default base delay of 100 milliseconds.
pub const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Default maximum timeout of 10 seconds.
pub const DEFAULT_MAX_TIMEOUT: Duration = Duration::from_seconds(10);

/// A default number of retries, 5.
pub const DEFAULT_RETRIES: u32 = 5;

pub use builder::BackoffBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backoff<C = u32> {
    max_retries: u32,
    retries: C,
    // opting to have these be u32s, rather than full on [`Duration`]s.
    // their added complexity isn't needed, and having to deal with the signed
    // output values to do math on is no fun.
    base_delay_ms: u32,
    max_timeout_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackoffOnce {
    max_retries: u32,
    on_retry: u32,
    waiting: Duration,
}

impl BackoffOnce {
    pub const fn on_retry(&self) -> u32 {
        self.on_retry
    }

    pub const fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

fn get_within_range(range: std::ops::RangeInclusive<u32>) -> u32 {
    rand::rng().random_range(range)
}

impl Default for Backoff {
    #[inline]
    fn default() -> Self {
        Self::new(DEFAULT_RETRIES, DEFAULT_BASE_DELAY, DEFAULT_MAX_TIMEOUT)
    }
}

impl<C: Default> Backoff<C> {
    #[inline]
    pub fn new(max_retries: u32, base_delay: Duration, max_timeout: Duration) -> Self {
        Self::new_inner(C::default(), max_retries, base_delay, max_timeout)
    }

    pub const fn builder() -> BackoffBuilder<C> {
        BackoffBuilder::new()
    }
}

impl<C> Backoff<C> {
    const fn new_inner(
        retries: C,
        max_retries: u32,
        base_delay: Duration,
        max_timeout: Duration,
    ) -> Self {
        const fn max_duration(dur: Duration, min: u32) -> u32 {
            let dur = dur.whole_milliseconds() as u32;

            if dur > min { dur } else { min }
        }

        Self {
            max_retries,
            retries,
            base_delay_ms: max_duration(base_delay, 25),
            max_timeout_ms: max_duration(max_timeout, 25),
        }
    }

    /// Returns 'true' if we've hit the maximum retry count.
    #[inline]
    pub const fn is_spent(&self, retries: u32) -> bool {
        retries > self.max_retries
    }

    #[inline]
    fn compute_backoff(&self, retries: u32) -> Duration {
        let slots = 2_u32.saturating_pow(retries);
        let full_delay_ms = slots * self.base_delay_ms;

        let sleep_ms = get_within_range(self.base_delay_ms..=full_delay_ms);

        Duration::from_millis(sleep_ms.min(self.max_timeout_ms) as _)
    }

    fn backoff_inner(&self, retries: u32) -> Option<BackoffOnce> {
        // bail if out of retries
        if self.is_spent(retries) {
            None
        } else {
            Some(BackoffOnce {
                on_retry: retries,
                max_retries: self.max_retries,
                waiting: self.compute_backoff(retries),
            })
        }
    }

    pub fn backoff_once<'a>(&'a mut self) -> Option<BackoffOnce>
    where
        Self: DoBackoff<Takes<'a> = &'a mut Self>,
    {
        <Self as DoBackoff>::backoff_once(self)
    }

    pub fn backoff_once_ref<'a>(&'a self) -> Option<BackoffOnce>
    where
        Self: DoBackoff<Takes<'a> = &'a Self>,
    {
        <Self as DoBackoff>::backoff_once(self)
    }
}

pub trait DoBackoff: Sized {
    type Takes<'a>
    where
        Self: 'a;

    fn backoff_once(takes: Self::Takes<'_>) -> Option<BackoffOnce>;
}

impl DoBackoff for Backoff<u32> {
    type Takes<'a> = &'a mut Self;
    #[inline]
    fn backoff_once(takes: &mut Self) -> Option<BackoffOnce> {
        takes.retries += 1;
        takes.backoff_inner(takes.retries)
    }
}

impl DoBackoff for Backoff<Arc<AtomicU32>> {
    type Takes<'a> = &'a Self;
    #[inline]
    fn backoff_once(takes: &Self) -> Option<BackoffOnce> {
        let retries = takes.retries.fetch_add(1, Ordering::SeqCst) + 1;
        takes.backoff_inner(retries)
    }
}

impl IntoFuture for BackoffOnce {
    type Output = ();
    type IntoFuture = Sleep;

    fn into_future(self) -> Self::IntoFuture {
        tokio::time::sleep(self.waiting.into())
    }
}

mod builder {
    use std::marker::PhantomData;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    pub struct BackoffBuilder<C> {
        max_retries: u32,
        base_delay: timestamp::Duration,
        max_timeout: timestamp::Duration,
        _marker: PhantomData<C>,
    }

    impl<C> BackoffBuilder<C> {
        pub const fn new() -> Self {
            Self {
                max_retries: super::DEFAULT_RETRIES,
                max_timeout: super::DEFAULT_MAX_TIMEOUT,
                base_delay: super::DEFAULT_BASE_DELAY,
                _marker: PhantomData,
            }
        }

        pub const fn max_retries(&mut self, max_retries: u32) -> &mut Self {
            self.max_retries = max_retries;
            self
        }

        pub const fn base_delay(&mut self, base_delay: timestamp::Duration) -> &mut Self {
            self.base_delay = base_delay;
            self
        }
    }

    impl BackoffBuilder<u32> {
        pub const fn build_local(&mut self) -> super::Backoff<u32> {
            super::Backoff::new_inner(0, self.max_retries, self.base_delay, self.max_timeout)
        }
    }

    impl BackoffBuilder<Arc<AtomicU32>> {
        pub fn build_shared(&mut self) -> super::Backoff<Arc<AtomicU32>> {
            super::Backoff::new_inner(
                Arc::new(AtomicU32::new(0)),
                self.max_retries,
                self.base_delay,
                self.max_timeout,
            )
        }
    }

    impl<C: Default> Default for BackoffBuilder<C> {
        fn default() -> Self {
            Self::new()
        }
    }
}
