use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::HeaderValue;
use timestamp::Timestamp;
use tokio::task::JoinHandle;

use crate::Scope;

pub(super) struct PendingRequest {
    // use a join handle, that way we can let other threads make progress on this.
    handle: Option<JoinHandle<crate::Result<(HeaderValue, Timestamp)>>>,
}

impl PendingRequest {
    pub(super) fn new(provider: &Arc<super::Provider>, scope: Scope) -> Self {
        let handle = tokio::spawn(provider.get_token(scope));
        Self {
            handle: Some(handle),
        }
    }

    pub(super) fn is_request_pending(&self) -> bool {
        self.handle.is_some()
    }

    pub(super) fn start_request(&mut self, provider: &Arc<super::Provider>, scope: Scope) {
        if let Some(old) = self.handle.replace(tokio::spawn(provider.get_token(scope))) {
            old.abort();
        }
    }

    pub(super) fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<(HeaderValue, Timestamp)>>> {
        let result = match self.handle {
            Some(ref mut handle) => std::task::ready!(Pin::new(handle).poll(cx)),
            None => return Poll::Ready(Ok(None)),
        };

        // if we finished, remove the handle
        self.handle = None;

        match dbg!(result) {
            Ok(result) => Poll::Ready(result.map(Some)),
            Err(error) if error.is_cancelled() => unreachable!("we never cancel these handles"),
            Err(error) => std::panic::resume_unwind(error.into_panic()),
        }
    }
}
