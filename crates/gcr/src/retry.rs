use std::fmt;
use std::future::Future;

use crate::backoff::{self, Backoff};

pub struct Retry<const MAX_RETRIES: u32, C, F> {
    backoff: Backoff<MAX_RETRIES>,
    classifier: C,
    retry_fn: F,
}

impl<const MAX_RETRIES: u32, C, F> Retry<MAX_RETRIES, C, F> {
    #[inline]
    pub const fn new(backoff: Backoff<MAX_RETRIES>, classifier: C, retry_fn: F) -> Self {
        Self {
            backoff,
            classifier,
            retry_fn,
        }
    }
}

impl<C, F> Retry<{ backoff::DEFAULT_RETRIES }, C, F> {
    #[inline]
    pub fn new_default(retry_fn: F) -> Self
    where
        C: Default,
    {
        Self {
            backoff: Backoff::default(),
            classifier: C::default(),
            retry_fn,
        }
    }
}

impl<C, F, Fut> Retry<{ backoff::DEFAULT_RETRIES }, C, F>
where
    F: FnMut() -> Fut,
    Fut: Future,
    C: Classify<Fut::Output>,
    C: Default,
{
    pub async fn run_default(retry_fn: F) -> (RetryResult, C::Output) {
        Self::new_default(retry_fn).run().await
    }
}

impl<const MAX_RETRIES: u32, C, F, Fut> Retry<MAX_RETRIES, C, F>
where
    F: FnMut() -> Fut,
    Fut: Future,
    C: Classify<Fut::Output>,
{
    pub async fn run(mut self) -> (RetryResult, C::Output) {
        macro_rules! try_once {
            ($self:ident; $dst:ident) => {
                let raw = ($self.retry_fn)().await;
                let (should_retry, res) = $self.classifier.classify(raw);

                match should_retry {
                    RetryResult::Error => return (should_retry, res),
                    RetryResult::Ok => return (should_retry, res),
                    RetryResult::ErrorRetry => $dst = res,
                }
            };
        }

        let mut result: C::Output;

        try_once!(self; result);

        while let Some((stats, backoff_wait)) = self.backoff.backoff_once() {
            self.classifier.log_on_retry(stats, &result);
            backoff_wait.await;

            try_once!(self; result);
        }

        (RetryResult::Error, result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RetryResult {
    Error = 0,
    ErrorRetry = 1,
    Ok = 2,
}

pub trait Classify<Response> {
    type Output: fmt::Debug;

    fn classify(&mut self, response: Response) -> (RetryResult, Self::Output);

    fn log_on_retry<const MAX_RETRIES: u32>(
        &self,
        stats: backoff::BackoffStats<MAX_RETRIES>,
        output: &Self::Output,
    ) {
        warn!(message = "task failed, retrying", ?stats, ?output);
    }
}

impl<R, O, F> Classify<R> for F
where
    F: FnMut(R) -> (RetryResult, O),
    O: fmt::Debug,
{
    type Output = O;

    fn classify(&mut self, response: R) -> (RetryResult, Self::Output) {
        (self)(response)
    }
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetryOn404;

#[cfg(feature = "reqwest")]
impl<E: fmt::Debug> Classify<Result<reqwest::Response, E>> for RetryOn404 {
    type Output = Result<reqwest::Response, E>;

    fn classify(&mut self, response: Result<reqwest::Response, E>) -> (RetryResult, Self::Output) {
        match response {
            Ok(resp) if resp.status().as_u16() == 404 => (RetryResult::ErrorRetry, Ok(resp)),
            Ok(resp) => (RetryResult::Ok, Ok(resp)),
            Err(err) => (RetryResult::Error, Err(err)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetryOn<F>(pub F);

impl<F, O> Classify<O> for RetryOn<F>
where
    F: FnMut(&O) -> RetryResult,
    O: fmt::Debug,
{
    type Output = O;

    fn classify(&mut self, response: O) -> (RetryResult, Self::Output) {
        ((self.0)(&response), response)
    }
}
