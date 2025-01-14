use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, PoisonError};

use protos::spanner::spanner_client::SpannerClient;
use protos::spanner::{self};
use tokio::sync::Notify;
use tokio::task::JoinSet;

use super::session::SessionKey;
use super::tracker::{Borrowed, SessionTracker, TryBorrowError};
use crate::client::ClientParts;

pub(super) struct SessionPoolInner {
    client: Arc<ClientParts>,
    nofify_returned: Notify,
    state: Mutex<State>,
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

impl SessionPoolInner {
    pub(super) fn new(client: Arc<ClientParts>) -> Self {
        Self {
            client,
            nofify_returned: Notify::new(),
            state: Mutex::new(State {
                sessions: SessionTracker::new(),
                next_key: SessionKey::MIN,
            }),
        }
    }

    pub(super) fn stats(&self) -> Stats {
        let guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        Stats {
            total: guard.sessions.total_sessions(),
            available: guard.sessions.available(),
        }
    }

    pub(super) fn try_borrow_session(
        self: &Arc<Self>,
    ) -> Result<super::BorrowedSession, TryBorrowError> {
        let session = self
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sessions
            .borrow()?;

        Ok(super::BorrowedSession::new(self, session))
    }

    /// Borrows a session, optionally with a timeout.
    ///
    /// Returns [`None`] if the pool is empty (i.e no sessions have been created yet, or we just
    /// deleted all of the sessions) or if the timeout is hit.
    pub(super) async fn borrow_session(
        self: &Arc<Self>,
        timeout: Option<timestamp::Duration>,
    ) -> Option<super::BorrowedSession> {
        match self.try_borrow_session() {
            Ok(session) => return Some(session),
            // no point in waiting if the pool is empty,
            Err(TryBorrowError::PoolEmpty) => return None,
            Err(TryBorrowError::AllBorrowed) => (),
        }

        let borrow_fut = async {
            loop {
                self.nofify_returned.notified().await;

                match self.try_borrow_session() {
                    Ok(session) => return Some(session),
                    // no point in waiting if the pool is empty,
                    Err(TryBorrowError::PoolEmpty) => return None,
                    Err(TryBorrowError::AllBorrowed) => (),
                }
            }
        };

        match timeout {
            Some(timeout) => tokio::time::timeout(timeout.into(), borrow_fut)
                .await
                .ok()
                .flatten(),
            None => borrow_fut.await,
        }
    }

    pub(crate) async fn get_or_create_session<'a>(
        self: &Arc<Self>,
        timeout: Option<timestamp::Duration>,
        batch_create: NonZeroUsize,
    ) -> crate::Result<super::BorrowedSession> {
        if let Some(session) = self.borrow_session(timeout).await {
            return Ok(session);
        }

        match batch_create.get() {
            0 => unreachable!("non-zero type"),
            1 => {
                let new_session = create_session(&self.client).await?;
                Ok(self.add_sessions([new_session]))
            }
            to_create => {
                let to_create = (to_create as u8).min(10);
                let sessions = batch_create_sessions(&self.client, to_create).await?;
                Ok(self.add_sessions(sessions))
            }
        }
    }

    fn add_sessions(
        self: &Arc<Self>,
        sessions: impl IntoIterator<Item = protos::spanner::Session>,
    ) -> super::BorrowedSession {
        let created = std::time::Instant::now();

        let mut new_session_iter = sessions.into_iter();

        let first_session = new_session_iter
            .next()
            .expect("we should always be given an iterator that creates at least 1 session");

        let mut guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        let session = guard.insert_new(created, Borrowed::Yes, first_session);
        let to_return = super::BorrowedSession::new(self, Arc::clone(session));

        for session in new_session_iter {
            guard.insert_new(created, Borrowed::No, session);
        }

        to_return
    }

    pub(super) fn return_session(&self, session: &super::session::Session) {
        let returned = self
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sessions
            .return_session(session);

        if returned {
            self.nofify_returned.notify_one();
        }
    }

    pub(super) fn delete_sessions(&self) -> Option<JoinSet<crate::Result<()>>> {
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

        let mut channel = self.client.channel.clone();
        let mut client = SpannerClient::new(&mut channel);

        for mut session in sessions {
            if let Some(raw) = session.close() {
                let delete_fut =
                    client.delete_session(spanner::DeleteSessionRequest { name: raw.name });

                set.spawn(async move {
                    delete_fut.await?;
                    Ok(())
                });
            }
        }

        if set.is_empty() { None } else { Some(set) }
    }
}

impl Drop for SessionPoolInner {
    fn drop(&mut self) {
        if let Some(mut session_delete_set) = self.delete_sessions() {
            tokio::spawn(async move {
                while let Some(result) = session_delete_set.join_next().await {
                    match result {
                        Ok(Ok(())) => (),
                        Ok(Err(error)) => {
                            tracing::warn!(message = "failed to delete session", ?error)
                        }
                        Err(join_error) => tracing::warn!(
                            message = "delete session task failed",
                            ?join_error,
                            is_panic = join_error.is_panic()
                        ),
                    }
                }
            });
        }
    }
}

impl State {
    fn insert_new(
        &mut self,
        created: std::time::Instant,
        borrowed: Borrowed,
        session: spanner::Session,
    ) -> &Arc<super::session::Session> {
        let key = self.next_key;
        self.next_key = self.next_key.next();

        let session = super::session::Session::new(created, key, session);

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
