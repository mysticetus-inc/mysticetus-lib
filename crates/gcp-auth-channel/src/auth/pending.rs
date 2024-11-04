use std::sync::Arc;
use std::task::{Context, Poll};

use gcp_auth::Token;
use tokio_util::sync::ReusableBoxFuture;

use crate::Scope;

pub(super) struct PendingRequest {
    future: ReusableBoxFuture<'static, Result<Arc<Token>, gcp_auth::Error>>,
    state: State,
}

enum State {
    Empty,
    Pending,
}

impl PendingRequest {
    pub(super) fn new(provider: Arc<dyn gcp_auth::TokenProvider>, scope: Scope) -> Self {
        Self {
            state: State::Pending,
            future: ReusableBoxFuture::new(async move { provider.token(&[scope.as_str()]).await }),
        }
    }

    pub(super) fn is_request_pending(&self) -> bool {
        matches!(self.state, State::Pending)
    }

    pub(super) fn start_request(
        &mut self,
        provider: Arc<dyn gcp_auth::TokenProvider>,
        scope: Scope,
    ) {
        self.future
            .set(async move { provider.token(&[scope.as_str()]).await });
        self.state = State::Pending;
    }

    pub(super) fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<Arc<Token>>, gcp_auth::Error>> {
        if matches!(self.state, State::Empty) {
            return Poll::Ready(Ok(None));
        }

        let res = std::task::ready!(self.future.poll(cx));
        self.state = State::Empty;
        Poll::Ready(res.map(Some))
    }
}
