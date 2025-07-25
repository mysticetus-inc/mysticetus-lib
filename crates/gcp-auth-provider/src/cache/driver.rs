use std::pin::Pin;
use std::task::{Context, Poll};

use net_utils::backoff::Backoff;
#[cfg(feature = "fair-mutex")]
use parking_lot::{FairMutex as Mutex, FairMutexGuard as MutexGuard};
#[cfg(not(feature = "fair-mutex"))]
use parking_lot::{Mutex, MutexGuard};
use tokio::task::{JoinError, JoinHandle};

use crate::providers::TokenProvider;
use crate::{GetTokenFuture, Token};

#[derive(Default)]
pub struct TokenRefresher {
    handle: Mutex<Option<JoinHandle<crate::Result<Token>>>>,
}

impl std::fmt::Debug for TokenRefresher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        enum State {
            Empty,
            RequestPending,
            RequestCompleted,
        }

        let mut dbg = f.debug_struct("TokenRefresher");

        let state = self.handle.try_lock().map(|guard| match *guard {
            Some(ref handle) if handle.is_finished() => State::RequestCompleted,
            Some(_) => State::RequestPending,
            None => State::Empty,
        });

        match state {
            Some(state) => dbg.field("state", &state).finish(),
            None => dbg.finish_non_exhaustive(),
        }
    }
}

impl TokenRefresher {
    pub(super) fn new_from_future(
        provider_name: &'static str,
        future: GetTokenFuture<'static>,
    ) -> Self {
        Self {
            handle: Mutex::new(Some(tokio::spawn(refresh_with_retries(
                provider_name,
                future,
            )))),
        }
    }

    pub fn new_with_refresh<T>(provider: &T) -> Self
    where
        T: TokenProvider + ?Sized,
    {
        Self {
            handle: Mutex::new(Some(tokio::spawn(refresh_with_retries(
                provider.name(),
                provider.get_token().into_static(),
            )))),
        }
    }

    pub fn lock(&self) -> TokenRefresherGuard<'_> {
        TokenRefresherGuard {
            guard: self.handle.lock(),
        }
    }
}

pub struct TokenRefresherGuard<'a> {
    guard: MutexGuard<'a, Option<JoinHandle<crate::Result<Token>>>>,
}

impl TokenRefresherGuard<'_> {
    pub fn force_start_request<T>(&mut self, provider: &T)
    where
        T: TokenProvider + ?Sized,
    {
        if let Some(old_request) = self.guard.replace(tokio::spawn(refresh_with_retries(
            provider.name(),
            provider.get_token().into_static(),
        ))) {
            old_request.abort();
        }
    }

    pub fn start_request<T>(&mut self, provider: &T)
    where
        T: TokenProvider + ?Sized,
    {
        if self.guard.is_none() {
            self.force_start_request(provider);
        }
    }

    pub fn poll_refresh<T>(
        mut self,
        cx: &mut Context<'_>,
        provider: &T,
    ) -> Poll<crate::Result<Token>>
    where
        T: TokenProvider + ?Sized,
    {
        loop {
            match *self.guard {
                Some(ref mut handle) => {
                    let join_result = std::task::ready!(Pin::new(handle).poll(cx));
                    *self.guard = None;
                    // drop the lock ASAP, that way we arent wasting cycles holding a lock
                    // while we inspect/convert result types
                    drop(self);

                    let result = unwrap_join_result(join_result);
                    return Poll::Ready(result.map_err(crate::Error::from));
                }
                None => {
                    *self.guard = Some(tokio::spawn(refresh_with_retries(
                        provider.name(),
                        provider.get_token().into_static(),
                    )));
                }
            }
        }
    }
}

fn unwrap_join_result<T>(join_result: Result<T, JoinError>) -> T {
    match join_result {
        Ok(inner) => inner,
        Err(error) if error.is_cancelled() => {
            unreachable!("we never cancel these handles")
        }
        Err(error) => std::panic::resume_unwind(error.into_panic()),
    }
}

async fn refresh_with_retries(
    provider_name: &'static str,
    future: GetTokenFuture<'static>,
) -> crate::Result<Token> {
    let mut request_future = std::pin::pin!(future);

    // do 1 request right off the bat, before constructing any
    // of the retry/backoff stuff, since the request is likely to succeed
    // first try.

    let mut last_error = match request_future.as_mut().await {
        Ok(token) => return Ok(token),
        Err(error) if error.is_fatal() => return Err(error),
        Err(error) => {
            request_future.as_mut().reset();
            error
        }
    };

    let mut backoff = Backoff::default();

    loop {
        let Some(backoff_once) = backoff.backoff_once() else {
            return Err(last_error);
        };

        tracing::warn!(
            message = "failed to refresh token, retrying after backoff...",
            provider = provider_name,
            error.display = %last_error,
            error = &last_error as &dyn std::error::Error,
            // offset both by 1, since we made an initial request
            on_retry = backoff_once.on_retry() + 1,
            max_retries = backoff_once.max_retries() + 1,
        );

        backoff_once.await;

        match request_future.as_mut().await {
            Ok(token) => return Ok(token),
            Err(error) if error.is_fatal() => return Err(error),
            Err(error) => {
                request_future.as_mut().reset();
                last_error = error;
            }
        }
    }
}
