use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::HeaderValue;
use timestamp::Timestamp;
use tokio::task::JoinHandle;

use crate::Scope;

pub(super) struct PendingRequest {
    // use a join handle, that way we can let other threads make progress on this.
    handle: JoinHandle<crate::Result<(HeaderValue, Timestamp)>>,
}

impl PendingRequest {
    pub(super) fn new(provider: &Arc<super::Provider>, scope: Scope) -> Self {
        let handle = tokio::spawn(provider.get_token(scope));
        Self { handle }
    }

    pub(super) fn is_request_pending(&self) -> bool {
        !self.handle.is_finished()
    }

    pub(super) fn start_request(&mut self, provider: &Arc<super::Provider>, scope: Scope) {
        if !self.handle.is_finished() {
            self.handle.abort();
        }

        let new_handle = tokio::spawn(provider.get_token(scope));
        let old_handle = std::mem::replace(&mut self.handle, new_handle);
        old_handle.abort();
    }

    pub(super) fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<(HeaderValue, Timestamp)>>> {
        if self.handle.is_finished() {
            return Poll::Ready(Ok(None));
        }

        match std::task::ready!(Pin::new(&mut self.handle).poll(cx)) {
            Ok(result) => Poll::Ready(result.map(Some)),
            Err(error) if error.is_cancelled() => unreachable!("we never cancel these handles"),
            Err(error) => std::panic::resume_unwind(error.into_panic()),
        }
    }
}
