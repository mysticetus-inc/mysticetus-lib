//! Internal type that keeps track of sessions, and whether or not they're borrowed.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use fxhash::{FxBuildHasher, FxHashMap};
use tokio::task::JoinHandle;

use super::session::{Session, SessionKey};
use crate::client::ClientParts;

#[derive(Default)]
pub(super) struct SessionTracker {
    sessions: FxHashMap<SessionKey, Arc<Session>>,
    // keeps sessions ordered by creation time, that way when we borrow one,
    // we borrow the oldest first.
    alive: BTreeSet<SessionKey>,
}

pub(super) enum TryBorrowError {
    PoolEmpty,
    AllBorrowed,
}

pub(super) enum ReturnSessionResult {
    Returned,
    Deleting(JoinHandle<crate::Result<()>>),
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Borrowed {
    Yes,
    No,
}

impl SessionTracker {
    pub(super) const fn new() -> Self {
        Self {
            sessions: FxHashMap::with_hasher(FxBuildHasher::new()),
            alive: BTreeSet::new(),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub(super) fn total_sessions(&self) -> usize {
        self.sessions.len()
    }

    pub(super) fn available(&self) -> usize {
        self.alive.len()
    }

    pub(super) fn borrow(&mut self) -> Result<Arc<Session>, TryBorrowError> {
        if self.sessions.is_empty() {
            return Err(TryBorrowError::PoolEmpty);
        }

        let key = self.alive.pop_first().ok_or(TryBorrowError::AllBorrowed)?;

        // this unwrap should be fine, since any key in 'alive' should also be in 'sessions'.
        Ok(Arc::clone(self.sessions.get(&key).unwrap()))
    }

    /// returns true if the session was actually returned
    pub(super) fn return_session(
        &mut self,
        parts: &Arc<ClientParts>,
        session: &Arc<Session>,
    ) -> ReturnSessionResult {
        match session.state(Ordering::Acquire) {
            super::session::SessionState::Alive => {
                self.alive.insert(session.key());
                ReturnSessionResult::Returned
            }
            super::session::SessionState::Deleted => {
                _ = self.sessions.remove(&session.key());
                ReturnSessionResult::Deleted
            }
            super::session::SessionState::PendingDeletion => {
                let session = self
                    .sessions
                    .remove(&session.key())
                    .unwrap_or_else(|| Arc::clone(session));

                let parts = Arc::clone(parts);

                ReturnSessionResult::Deleting(tokio::spawn(
                    async move { session.delete(&parts).await },
                ))
            }
        }
    }

    pub(super) fn insert(&mut self, borrowed: Borrowed, session: Arc<Session>) -> &Arc<Session> {
        let key = session.key();

        if matches!(borrowed, Borrowed::No) {
            self.alive.insert(key);
        }

        self.sessions.entry(key).insert_entry(session).into_mut()
    }
}

impl IntoIterator for SessionTracker {
    type Item = Arc<Session>;
    type IntoIter = std::collections::hash_map::IntoValues<SessionKey, Arc<Session>>;

    fn into_iter(self) -> Self::IntoIter {
        self.sessions.into_values()
    }
}
