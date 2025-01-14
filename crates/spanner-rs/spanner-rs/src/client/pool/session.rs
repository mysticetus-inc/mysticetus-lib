use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::{PoisonError, RwLock};
use std::time::Instant;

use protos::spanner;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(super) struct SessionKey(NonZeroUsize);

impl SessionKey {
    pub(super) const MIN: Self = Self(NonZeroUsize::MIN);

    pub(super) fn next(self) -> Self {
        Self(self.0.checked_add(1).expect("session key overflow"))
    }
}

#[derive(Debug)]
pub struct Session {
    state: RwLock<SessionState>,
    created: Instant,
    last_used: AtomicU64,
    key: SessionKey,
}

impl Session {
    pub(super) fn close(&self) -> Option<Box<spanner::Session>> {
        self.state
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .close()
    }

    pub(super) fn is_closed(&self) -> bool {
        self.state
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .raw_session()
            .is_none()
    }

    pub(super) fn key(&self) -> SessionKey {
        self.key
    }

    pub(super) fn new(created: Instant, key: SessionKey, raw_session: spanner::Session) -> Self {
        Self {
            created,
            key,
            last_used: AtomicU64::new(0),
            state: RwLock::new(SessionState::Live(Box::new(raw_session))),
        }
    }
}

#[derive(Debug)]
enum SessionState {
    Live(Box<spanner::Session>),
    Closed,
}

impl SessionState {
    fn raw_session(&self) -> Option<&spanner::Session> {
        match self {
            Self::Live(raw) => Some(raw),
            Self::Closed => None,
        }
    }

    pub fn close(&mut self) -> Option<Box<spanner::Session>> {
        match std::mem::replace(self, Self::Closed) {
            Self::Live(session) => Some(session),
            Self::Closed => None,
        }
    }
}
