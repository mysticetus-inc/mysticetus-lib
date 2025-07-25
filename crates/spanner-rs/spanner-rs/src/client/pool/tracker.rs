//! Internal type that keeps track of sessions, and whether or not they're borrowed.

use std::collections::BTreeSet;
use std::hash::BuildHasher;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use hashbrown::hash_table::OccupiedEntry;
use tokio::task::JoinHandle;

use super::session::Session;
use crate::client::ClientParts;
use crate::client::pool::session::SessionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct SessionKey {
    added: NonZeroUsize,
    index: usize,
    name_hash: u64,
}

pub(super) struct SessionTracker {
    total_added: NonZeroUsize,
    sessions: slab::Slab<Arc<Session>>,
    // Maps the hash of the session name to the index
    mapping: hashbrown::HashTable<SessionKey>,
    name_hasher: fxhash::FxBuildHasher,
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
            total_added: NonZeroUsize::MIN,
            sessions: slab::Slab::new(),
            name_hasher: fxhash::FxBuildHasher::new(),
            mapping: hashbrown::HashTable::new(),
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
        Ok(Arc::clone(self.sessions.get(key.index).unwrap()))
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
                _ = self.sessions.remove(session.key().index);
                ReturnSessionResult::Deleted
            }
            super::session::SessionState::PendingDeletion => {
                let session = self
                    .sessions
                    .try_remove(session.key().index)
                    .unwrap_or_else(|| Arc::clone(session));

                let parts = Arc::clone(parts);

                ReturnSessionResult::Deleting(tokio::spawn(
                    async move { session.delete(&parts).await },
                ))
            }
        }
    }

    pub(super) fn insert_recovered(
        &mut self,
        created: Instant,
        raw: protos::spanner::Session,
    ) -> bool {
        let key = SessionKey {
            added: self.total_added,
            name_hash: self.name_hasher.hash_one(&raw.name),
            index: self.sessions.vacant_key(),
        };

        if self.try_insert_key(key).is_err() {
            return false;
        }

        // increment only once we know we've added a unique session
        self.total_added = self
            .total_added
            .checked_add(1)
            .expect("session added overflowed usize::MAX");

        let session = Session::new(created, key, raw);

        let idx = self.sessions.insert(Arc::new(session));

        debug_assert_eq!(idx, key.index);

        self.alive.insert(key);

        true
    }

    fn try_insert_key(&mut self, key: SessionKey) -> Result<(), OccupiedEntry<'_, SessionKey>> {
        match self
            .mapping
            .entry(key.name_hash, |k| k == &key, |k| k.name_hash)
        {
            hashbrown::hash_table::Entry::Vacant(vac) => {
                vac.insert(key);
                Ok(())
            }
            hashbrown::hash_table::Entry::Occupied(occ) => {
                if let Some(session) = self.sessions.get(occ.get().index) {
                    if session.state(Ordering::Relaxed) != SessionState::Deleted {
                        return Err(occ);
                    }
                }

                *occ.into_mut() = key;
                Ok(())
            }
        }
    }

    pub(super) fn insert_created(
        &mut self,
        borrowed: Borrowed,
        created: Instant,
        raw: protos::spanner::Session,
    ) -> &Arc<Session> {
        let key = SessionKey {
            added: self.total_added,
            name_hash: self.name_hasher.hash_one(&raw.name),
            index: self.sessions.vacant_key(),
        };

        let session = Session::new(created, key, raw);

        match self.try_insert_key(key) {
            Ok(()) => {
                self.total_added = self
                    .total_added
                    .checked_add(1)
                    .expect("session added overflowed usize::MAX");

                let idx = self.sessions.insert(Arc::new(session));

                debug_assert_eq!(idx, key.index);

                if matches!(borrowed, Borrowed::No) {
                    self.alive.insert(key);
                }

                self.sessions.get(idx).expect("just inserted it")
            }
            Err(occ) => {
                let existing_index = occ.into_mut().index;
                &self.sessions[existing_index]
            }
        }
    }
}

impl IntoIterator for SessionTracker {
    type Item = Arc<Session>;
    type IntoIter =
        std::iter::Map<slab::IntoIter<Arc<Session>>, fn((usize, Arc<Session>)) -> Arc<Session>>;

    fn into_iter(self) -> Self::IntoIter {
        self.sessions.into_iter().map(|(_, sess)| sess)
    }
}
