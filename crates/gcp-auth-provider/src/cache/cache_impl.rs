use std::task::{Context, Poll};

use futures::future::TryMaybeDone;
use parking_lot::RwLock;

use super::{TokenState, ValidAuth};
use crate::providers::TokenProvider;
use crate::{GetTokenFuture, ProjectId, Token};

#[derive(Debug)]
pub struct CachedTokenProvider<T: TokenProvider> {
    provider: T,
    project_id: ProjectId,
    state: RwLock<TokenState>,
}

impl<T: TokenProvider> CachedTokenProvider<T> {
    pub fn new(
        provider: T,
        project_id: ProjectId,
        token_fut: TryMaybeDone<GetTokenFuture<'static>>,
    ) -> Self {
        let state = RwLock::new(super::TokenState::new_from_token_fut(token_fut, &provider));

        Self {
            provider,
            project_id,
            state,
        }
    }
}

impl<T: TokenProvider + Send + Sync + 'static> super::TokenCache for CachedTokenProvider<T> {
    #[inline]
    fn revoke(&self, start_new_request: super::StartNewRequestOnRevoke) {
        let mut guard = self.state.write();
        guard.cached = None;

        if matches!(start_new_request, super::StartNewRequestOnRevoke::Yes) {
            guard.refresher.lock().start_request(&self.provider);
        }
    }

    #[inline]
    fn provider_name(&self) -> &'static str {
        self.provider.name()
    }

    #[inline]
    fn project_id(&self) -> ProjectId {
        self.project_id
    }

    #[inline]
    fn get_cached_header(&self) -> Option<ValidAuth> {
        self.state.read().get_valid_auth()
    }

    #[inline]
    fn poll_refresh(&self, cx: &mut Context<'_>) -> Poll<crate::Result<ValidAuth>> {
        let Some(mut guard) = self
            .state
            .try_write_for(std::time::Duration::from_millis(100))
        else {
            // register another wakeup to try again,
            // but yield to the runtime so we dont block waiting
            // for a lock forever
            cx.waker().wake_by_ref();
            return Poll::Pending;
        };

        // another thread might have finished the request, so we need to check for a valid token
        // right after we get the lock.
        if let Some(valid_auth) = guard.get_valid_auth() {
            return Poll::Ready(Ok(valid_auth));
        }

        let refresher = guard.refresher.lock();
        let token = std::task::ready!(refresher.poll_refresh(cx, &self.provider))?;

        // we dont need a unique reference, so downcast to a shared ref
        let token: &Token = guard.cached.insert(token);

        match token.valid_for() {
            Ok(valid_for) => Poll::Ready(Ok(ValidAuth {
                valid_for,
                header: token.header().clone(),
            })),
            Err(()) => panic!("brand new token from request already expired?"),
        }
    }
}
