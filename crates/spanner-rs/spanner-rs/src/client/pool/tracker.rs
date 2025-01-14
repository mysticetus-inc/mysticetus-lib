//! Internal type that keeps track of sessions, and whether or not they're borrowed.

use std::collections::BTreeSet;
use std::sync::Arc;

use fxhash::{FxBuildHasher, FxHashMap};

use super::session::{Session, SessionKey};

#[derive(Default)]
pub(super) struct SessionTracker {
    sessions: FxHashMap<SessionKey, Arc<Session>>,
    alive: BTreeSet<SessionKey>,
}

pub(super) enum TryBorrowError {
    PoolEmpty,
    AllBorrowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Borrowed {
    Yes,
    No,
}

impl SessionTracker {
    pub(super) fn new() -> Self {
        Self {
            sessions: FxHashMap::with_capacity_and_hasher(32, FxBuildHasher::default()),
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
    pub(super) fn return_session(&mut self, session: &Session) -> bool {
        // if the session is closed, remove it entirely
        if session.is_closed() {
            let _ = self.sessions.remove(&session.key());
            false
        } else {
            self.alive.insert(session.key())
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
