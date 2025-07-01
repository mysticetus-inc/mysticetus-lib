use std::sync::Arc;
use std::task::{Context, Poll};

use gcp_auth::Token;
use http::HeaderValue;
use timestamp::Timestamp;
use tokio_util::sync::ReusableBoxFuture;

use crate::Scope;

pub(super) struct PendingRequest {
    future: ReusableBoxFuture<'static, crate::Result<(HeaderValue, Timestamp)>>,
    state: State,
}

enum State {
    Empty,
    Pending,
}

impl PendingRequest {
    pub(super) fn new(provider: &Arc<super::Provider>, scope: Scope) -> Self {
        let future = provider.get_token(scope);
        Self {
            state: State::Pending,
            future: ReusableBoxFuture::new(future),
        }
    }

    pub(super) fn is_request_pending(&self) -> bool {
        matches!(self.state, State::Pending)
    }

    pub(super) fn start_request(&mut self, provider: &Arc<super::Provider>, scope: Scope) {
        self.future.set(provider.get_token(scope));
        self.state = State::Pending;
    }

    pub(super) fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<(HeaderValue, Timestamp)>>> {
        if matches!(self.state, State::Empty) {
            return Poll::Ready(Ok(None));
        }

        let res = std::task::ready!(self.future.poll(cx));
        self.state = State::Empty;
        Poll::Ready(res.map(Some))
    }
}
