use std::future::Future;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use timestamp::Duration;

/// Default base delay of 100 milliseconds.
pub const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Default maximum timeout of 10 seconds.
pub const DEFAULT_MAX_TIMEOUT: Duration = Duration::from_seconds(10);

/// A default number of retries, 5.
pub const DEFAULT_RETRIES: u32 = 5;

#[derive(Debug, PartialEq, Eq)]
pub struct Backoff<const MAX_RETRIES: u32> {
    /// most request will succeed the first time, so don't initialize an RNG until
    /// we need it, via [`get_or_init_rng`].
    rng: Option<SmallRng>,
    retries: u32,
    // opting to have these be u32s, rather than full on [`Duration`]s.
    // their added complexity isn't needed, and having to deal with the signed
    // output values to do math on is no fun.
    base_delay_ms: u32,
    max_timeout_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackoffStats<const MAX_RETRIES: u32> {
    on_retry: u32,
    waiting: Duration,
}

fn get_or_init_rng(rng: &mut Option<SmallRng>) -> &mut SmallRng {
    rng.get_or_insert_with(|| SmallRng::from_rng(&mut rand::rng()))
}

impl Default for Backoff<DEFAULT_RETRIES> {
    #[inline]
    fn default() -> Self {
        Self::new(DEFAULT_BASE_DELAY, DEFAULT_MAX_TIMEOUT)
    }
}

impl<const MAX_RETRIES: u32> Backoff<MAX_RETRIES> {
    #[inline]
    pub const fn new(base_delay: Duration, max_timeout: Duration) -> Self {
        const fn const_max(a: u32, b: u32) -> u32 {
            if a > b { a } else { b }
        }

        Self {
            rng: None,
            retries: 0,
            base_delay_ms: const_max(base_delay.whole_milliseconds() as u32, 25),
            max_timeout_ms: const_max(max_timeout.whole_milliseconds() as u32, 25),
        }
    }

    /// Returns 'true' if we've hit the maximum retry count.
    #[inline]
    pub const fn is_spent(&self) -> bool {
        self.retries > MAX_RETRIES
    }

    pub fn backoff_once(
        &mut self,
    ) -> Option<(
        BackoffStats<MAX_RETRIES>,
        impl Future<Output = ()> + Send + Sync + 'static,
    )> {
        self.retries = self.retries.saturating_add(1);

        // bail if out of retries
        if self.is_spent() {
            return None;
        }

        let slots = 2_u32.saturating_pow(self.retries);
        let full_delay_ms = slots * self.base_delay_ms;

        let sleep_ms =
            get_or_init_rng(&mut self.rng).random_range(self.base_delay_ms..=full_delay_ms);

        let waiting = Duration::from_millis(sleep_ms.min(self.max_timeout_ms) as _);

        let stats = BackoffStats {
            on_retry: self.retries,
            waiting,
        };

        Some((stats, tokio::time::sleep(stats.waiting.into())))
    }
}
