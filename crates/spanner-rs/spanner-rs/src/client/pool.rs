use std::fmt;
use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, Weak};

use protos::spanner::{self};

mod internal;
mod session;
mod tracker;

use crate::client::ClientParts;

static POOL_DEBUG: LazyLock<bool> = LazyLock::new(|| std::env::var("POOL_DEBUG").is_ok());

pub const MAX_SESSION_COUNT: u8 = 100;

#[derive(Clone)]
pub struct SessionPool {
    inner: Arc<internal::SessionPoolInner>,
}

impl fmt::Debug for SessionPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let internal::Stats { total, available } = self.inner.stats();

        f.debug_struct("SessionPool")
            .field("total", &total)
            .field("available", &available)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum PoolError {
    #[error("session pool is full")]
    SessionPoolFull,
    #[error("timed out waiting for session to become available")]
    Timeout,
}

impl SessionPool {
    pub(crate) fn new(client: Arc<ClientParts>) -> Self {
        Self {
            inner: Arc::new(internal::SessionPoolInner::new(client)),
        }
    }

    async fn borrow_session(
        &self,
        timeout: Option<timestamp::Duration>,
    ) -> Option<BorrowedSession> {
        self.inner.borrow_session(timeout).await
    }

    async fn get_or_create_session<'a>(
        &self,
        timeout: Option<timestamp::Duration>,
        batch_create: NonZeroUsize,
    ) -> crate::Result<BorrowedSession> {
        self.inner
            .get_or_create_session(timeout, batch_create)
            .await
    }
}

pub struct BorrowedSession {
    pool: Weak<internal::SessionPoolInner>,
    session: Arc<session::Session>,
}

impl BorrowedSession {
    fn new(pool: &Arc<internal::SessionPoolInner>, session: Arc<session::Session>) -> Self {
        Self {
            pool: Arc::downgrade(pool),
            session,
        }
    }

    fn close(&self) -> Option<Box<spanner::Session>> {
        self.session.close()
    }
}

impl Drop for BorrowedSession {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.upgrade() {
            pool.return_session(&self.session);
        }
    }
}
