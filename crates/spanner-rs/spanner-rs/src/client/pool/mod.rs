use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use protos::spanner::spanner_client::SpannerClient;
use protos::spanner::{self};
use tokio::sync::Notify;
use tokio::task::{JoinHandle, JoinSet};

static POOL_DEBUG: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("SPANNER_POOL_DEBUG").is_ok_and(|value| !value.is_empty() && value != "0")
});

mod session;
mod shutdown;
mod tracker;

pub use session::Session;
use session::SessionKey;
use tracker::{Borrowed, SessionTracker, TryBorrowError};

use crate::client::ClientParts;

pub const MAX_SESSION_COUNT: u8 = 100;

/// Static session pool that all clients use.
pub(crate) static SESSION_POOL: SessionPool = SessionPool {
    closed: AtomicBool::new(false),
    nofify_returned: Notify::const_new(),
    state: Mutex::new(State {
        sessions: SessionTracker::new(),
        next_key: SessionKey::MIN,
    }),
};

pub(crate) struct SessionPool {
    closed: AtomicBool,
    nofify_returned: Notify,
    state: Mutex<State>,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum PoolError {
    #[error("session pool is full")]
    SessionPoolFull,
    #[error("timed out waiting for session to become available")]
    Timeout,
    #[error("pool is closed pending shutdown")]
    PoolClosed,
}

struct State {
    sessions: SessionTracker,
    next_key: SessionKey,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Stats {
    pub(super) total: usize,
    pub(super) available: usize,
}

impl SessionPool {
    pub(super) fn stats() -> Stats {
        let guard = SESSION_POOL
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        Stats {
            total: guard.sessions.total_sessions(),
            available: guard.sessions.available(),
        }
    }

    fn try_borrow_session(&self) -> Result<Result<Arc<Session>, TryBorrowError>, PoolError> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(PoolError::PoolClosed);
        }

        Ok(self
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sessions
            .borrow())
    }

    /// Borrows a session, optionally with a timeout.
    ///
    /// Returns [`None`] if the pool is empty (i.e no sessions have been created yet, or we just
    /// deleted all of the sessions) or if the timeout is hit.
    async fn borrow_session(
        &self,
        timeout: Option<timestamp::Duration>,
    ) -> Result<Option<Arc<Session>>, PoolError> {
        match self.try_borrow_session()? {
            Ok(session) => return Ok(Some(session)),
            // no point in waiting if the pool is empty,
            Err(TryBorrowError::PoolEmpty) => return Ok(None),
            Err(TryBorrowError::AllBorrowed) => (),
        }

        let borrow_fut = async {
            loop {
                self.nofify_returned.notified().await;

                match self.try_borrow_session()? {
                    Ok(session) => return Ok(Some(session)),
                    // no point in waiting if the pool is empty,
                    Err(TryBorrowError::PoolEmpty) => return Ok(None),
                    Err(TryBorrowError::AllBorrowed) => (),
                }
            }
        };

        match timeout {
            Some(timeout) => tokio::time::timeout(timeout.into(), borrow_fut)
                .await
                .ok()
                .transpose()
                .map(Option::flatten),
            None => borrow_fut.await,
        }
    }

    async fn get_or_create_session<'a>(
        &self,
        client: &ClientParts,
        timeout: Option<timestamp::Duration>,
        batch_create: NonZeroUsize,
    ) -> crate::Result<Arc<Session>> {
        if let Some(session) = self.borrow_session(timeout).await? {
            return Ok(session);
        }

        match batch_create.get() {
            0 => unreachable!("non-zero type"),
            1 => {
                let new_session = create_session(&client).await?;
                Ok(self.add_sessions([new_session]))
            }
            to_create => {
                let to_create = (to_create as u8).min(10);
                let sessions = batch_create_sessions(&client, to_create).await?;
                Ok(self.add_sessions(sessions))
            }
        }
    }

    pub async fn get_or_create_session_according_to_load<'a>(
        client: &ClientParts,
        timeout: Option<timestamp::Duration>,
    ) -> crate::Result<Arc<Session>> {
        if let Some(session) = SESSION_POOL.borrow_session(timeout).await? {
            return Ok(session);
        }

        let stats = Self::stats();

        let to_create = (stats.total / 2).clamp(1, 5);

        match to_create {
            0 => unreachable!("clamped to 1-5"),
            1 => {
                let new_session = create_session(client).await?;
                Ok(SESSION_POOL.add_sessions([new_session]))
            }
            to_create => {
                let to_create = (to_create as u8).min(10);
                let sessions = batch_create_sessions(client, to_create).await?;
                Ok(SESSION_POOL.add_sessions(sessions))
            }
        }
    }

    fn add_sessions(
        &self,
        sessions: impl IntoIterator<Item = protos::spanner::Session>,
    ) -> Arc<Session> {
        let created = std::time::Instant::now();

        let mut new_session_iter = sessions.into_iter();

        let first_session = new_session_iter
            .next()
            .expect("we should always be given an iterator that creates at least 1 session");

        let mut guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        let session = guard.insert_new(created, Borrowed::Yes, first_session);
        let to_return = Arc::clone(session);

        for session in new_session_iter {
            guard.insert_new(created, Borrowed::No, session);
        }

        to_return
    }

    pub(super) fn return_session(
        &self,
        parts: &Arc<ClientParts>,
        session: &Arc<Session>,
    ) -> Option<JoinHandle<crate::Result<()>>> {
        match self
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sessions
            .return_session(parts, session)
        {
            tracker::ReturnSessionResult::Deleted => None,
            tracker::ReturnSessionResult::Deleting(handle) => Some(handle),
            tracker::ReturnSessionResult::Returned => {
                self.nofify_returned.notify_one();
                None
            }
        }
    }

    pub(super) fn delete_sessions(
        &self,
        parts: &Arc<ClientParts>,
    ) -> Option<JoinSet<crate::Result<()>>> {
        self.closed.store(true, Ordering::SeqCst);

        let sessions = {
            let mut guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            std::mem::take(&mut guard.sessions)
        };

        // notify everything waiting so they can error out instead of hang
        self.nofify_returned.notify_waiters();

        if sessions.is_empty() {
            return None;
        }

        let mut set = JoinSet::new();

        for session in sessions {
            let parts = parts.clone();
            set.spawn(async move { session.delete(&parts).await });
        }

        if set.is_empty() { None } else { Some(set) }
    }
}

impl State {
    fn insert_new(
        &mut self,
        created: std::time::Instant,
        borrowed: Borrowed,
        session: spanner::Session,
    ) -> &Arc<Session> {
        let key = self.next_key;
        self.next_key = self.next_key.next();

        let session = Session::new(created, key, session);

        self.sessions.insert(borrowed, Arc::new(session))
    }
}

async fn create_session(parts: &ClientParts) -> crate::Result<spanner::Session> {
    SpannerClient::new(parts.channel.clone())
        .create_session(spanner::CreateSessionRequest {
            database: parts.info.qualified_database().to_owned(),
            session: parts.role.as_deref().map(|role| spanner::Session {
                creator_role: role.to_owned(),
                ..Default::default()
            }),
        })
        .await
        .map(tonic::Response::into_inner)
        .map_err(crate::Error::Status)
}

async fn batch_create_sessions(
    parts: &ClientParts,
    to_create: u8,
) -> crate::Result<Vec<spanner::Session>> {
    debug_assert_ne!(
        to_create, 0,
        "we should never be trying to batch create 0 sessions"
    );

    SpannerClient::new(parts.channel.clone())
        .batch_create_sessions(spanner::BatchCreateSessionsRequest {
            database: parts.info.qualified_database().to_owned(),
            session_count: to_create as i32,
            session_template: parts.role.as_deref().map(|role| spanner::Session {
                creator_role: role.to_owned(),
                ..Default::default()
            }),
        })
        .await
        .map(|resp| resp.into_inner().session)
        .map_err(crate::Error::Status)
}
