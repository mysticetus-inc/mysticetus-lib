//! A universal exponential backoff mechanism.

use std::future::IntoFuture;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use timestamp::Duration;
use tokio::time::{Instant, Sleep};

/// Default base delay of 100 milliseconds.
pub const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Default maximum timeout of 10 seconds.
pub const DEFAULT_MAX_TIMEOUT: Duration = Duration::from_seconds(10);

/// A default number of retries, 5.
pub const DEFAULT_RETRIES: u32 = 5;

pub use builder::BackoffBuilder;

/// Config for a [Backoff].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackoffConfig {
    max_retries: u32,
    // opting to have these be u32s, rather than full on [`Duration`]s.
    // their added complexity isn't needed, and having to deal with the signed
    // output values to do math on is no fun.
    base_delay_ms: u32,
    max_timeout_ms: u32,
}

impl BackoffConfig {
    pub const fn new(max_retries: u32, base_delay: Duration, max_timeout: Duration) -> Self {
        const fn max_duration(dur: Duration, min: u32) -> u32 {
            let dur = dur.whole_milliseconds() as u32;

            if dur > min { dur } else { min }
        }

        Self {
            max_retries,
            base_delay_ms: max_duration(base_delay, 25),
            max_timeout_ms: max_duration(max_timeout, 25),
        }
    }

    #[inline]
    fn compute_backoff<R>(&self, retries: u32, rng: &mut R) -> Duration
    where
        R: Rng + ?Sized,
    {
        let slots = 2_u32.saturating_pow(retries);
        let full_delay_ms = slots * self.base_delay_ms;

        let sleep_ms = rng.random_range(self.base_delay_ms..=full_delay_ms);

        Duration::from_millis(sleep_ms.min(self.max_timeout_ms) as _)
    }

    #[inline]
    const fn is_spent(&self, retries: u32) -> bool {
        retries > self.max_retries
    }

    pub fn make_backoff<C: Default, R: SeedableRng>(&self) -> Backoff<C, R> {
        Backoff {
            config: *self,
            rng: R::from_os_rng(),
            retries: C::default(),
        }
    }
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self::new(DEFAULT_RETRIES, DEFAULT_BASE_DELAY, DEFAULT_MAX_TIMEOUT)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backoff<C = u32, R = SmallRng> {
    retries: C,
    rng: R,
    config: BackoffConfig,
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

    pub fn waiting(&self) -> Duration {
        self.waiting
    }

    pub fn reset_existing(self, sleep: std::pin::Pin<&mut tokio::time::Sleep>) {
        sleep.reset(Instant::now() + self.waiting.into());
    }

    pub fn insert_or_reset(
        self,
        sleep_opt: &mut Option<std::pin::Pin<Box<tokio::time::Sleep>>>,
    ) -> std::pin::Pin<&mut tokio::time::Sleep> {
        match sleep_opt {
            Some(sleep) => {
                self.reset_existing(sleep.as_mut());
                sleep.as_mut()
            }
            None => sleep_opt.insert(Box::pin(self.into_future())).as_mut(),
        }
    }
}

impl Default for Backoff {
    #[inline]
    fn default() -> Self {
        Self::new(DEFAULT_RETRIES, DEFAULT_BASE_DELAY, DEFAULT_MAX_TIMEOUT)
    }
}

impl<C: Default, R: Rng + SeedableRng> Backoff<C, R> {
    #[inline]
    pub fn new(max_retries: u32, base_delay: Duration, max_timeout: Duration) -> Self {
        Self::new_inner(
            C::default(),
            R::from_os_rng(),
            max_retries,
            base_delay,
            max_timeout,
        )
    }

    pub const fn builder() -> BackoffBuilder<C> {
        BackoffBuilder::new()
    }
}

impl Backoff<u32> {
    pub const fn reset(&mut self) {
        self.retries = 0;
    }
}

impl Backoff<AtomicU32> {
    pub fn reset(&self, order: Ordering) {
        self.retries.store(0, order);
    }
}

impl<C, R> Backoff<C, R> {
    const fn new_inner(
        retries: C,
        rng: R,
        max_retries: u32,
        base_delay: Duration,
        max_timeout: Duration,
    ) -> Self {
        Self {
            retries,
            rng,
            config: BackoffConfig::new(max_retries, base_delay, max_timeout),
        }
    }

    #[inline]
    pub const fn retries(&self) -> C
    where
        C: Copy,
    {
        self.retries
    }

    #[inline]
    pub const fn config(&self) -> BackoffConfig {
        self.config
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

fn backoff_once_inner<R: Rng + ?Sized>(
    config: &BackoffConfig,
    retries: u32,
    rng: &mut R,
) -> Option<BackoffOnce> {
    if config.is_spent(retries) {
        None
    } else {
        Some(BackoffOnce {
            on_retry: retries,
            max_retries: config.max_retries,
            waiting: config.compute_backoff(retries, rng),
        })
    }
}

impl DoBackoff for Backoff<u32> {
    type Takes<'a> = &'a mut Self;
    #[inline]
    fn backoff_once(takes: &mut Self) -> Option<BackoffOnce> {
        takes.retries += 1;
        backoff_once_inner(&takes.config, takes.retries, &mut takes.rng)
    }
}

impl DoBackoff for Backoff<Arc<AtomicU32>, ()> {
    type Takes<'a> = &'a Self;
    #[inline]
    fn backoff_once(takes: &Self) -> Option<BackoffOnce> {
        let retries = takes.retries.fetch_add(1, Ordering::SeqCst) + 1;
        backoff_once_inner(&takes.config, retries, &mut rand::rng())
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

    use rand::SeedableRng;
    use rand::rngs::SmallRng;

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

        pub const fn max_timeout(&mut self, max_timeout: timestamp::Duration) -> &mut Self {
            self.max_timeout = max_timeout;
            self
        }

        pub const fn base_delay(&mut self, base_delay: timestamp::Duration) -> &mut Self {
            self.base_delay = base_delay;
            self
        }
    }

    impl BackoffBuilder<u32> {
        pub fn build_local(&mut self) -> super::Backoff<u32> {
            super::Backoff::new_inner(
                0,
                SmallRng::from_os_rng(),
                self.max_retries,
                self.base_delay,
                self.max_timeout,
            )
        }
    }

    impl BackoffBuilder<Arc<AtomicU32>> {
        pub fn build_shared(&mut self) -> super::Backoff<Arc<AtomicU32>, ()> {
            super::Backoff::new_inner(
                Arc::new(AtomicU32::new(0)),
                (),
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
