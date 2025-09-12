use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

use protos::spanner::spanner_client::SpannerClient;
use protos::spanner::{self, DeleteSessionRequest};

use crate::client::ClientParts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(super) struct SessionKey(NonZeroUsize);

impl SessionKey {
    pub(super) const MIN: Self = Self(NonZeroUsize::MIN);

    pub(super) fn next(self) -> Self {
        Self(self.0.checked_add(1).expect("session key overflow"))
    }
}

const SESSION_ALIVE: u8 = 0;
const SESSION_PENDING_DELETION: u8 = 1;
const SESSION_DELETED: u8 = 2;

#[derive(Debug)]
pub struct Session {
    raw: spanner::Session,
    state: AtomicU8,
    created: Instant,
    last_used: AtomicU64,
    key: SessionKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Alive,
    PendingDeletion,
    Deleted,
}

impl Session {
    pub(super) fn state(&self, order: Ordering) -> SessionState {
        match self.state.load(order) {
            SESSION_ALIVE => SessionState::Alive,
            SESSION_PENDING_DELETION => SessionState::PendingDeletion,
            SESSION_DELETED => SessionState::Deleted,
            _ => unreachable!(),
        }
    }

    pub(super) fn alive(&self) -> bool {
        self.state.load(Ordering::Acquire) == SESSION_ALIVE
    }

    pub(super) fn pending_deletion(&self) -> bool {
        self.state.load(Ordering::Acquire) == SESSION_PENDING_DELETION
    }

    pub(super) fn mark_for_deletion(&self) {
        _ = self.state.compare_exchange_weak(
            SESSION_ALIVE,
            SESSION_PENDING_DELETION,
            Ordering::SeqCst,
            Ordering::Relaxed,
        );
    }

    pub(crate) async fn delete(&self, parts: &ClientParts) -> crate::Result<()> {
        self.mark_for_deletion();

        if self.pending_deletion() {
            SpannerClient::new(parts.channel.clone())
                .delete_session(DeleteSessionRequest {
                    name: self.raw.name.clone(),
                })
                .await?;

            self.state.store(SESSION_DELETED, Ordering::Release);
        }

        Ok(())
    }

    pub(crate) fn raw_session(&self) -> Option<&protos::spanner::Session> {
        if self.alive() { Some(&self.raw) } else { None }
    }

    pub(crate) fn raw_session_name(&self) -> Option<String> {
        self.raw_session()
            .map(|raw_session| raw_session.name.clone())
    }

    pub(super) fn key(&self) -> SessionKey {
        self.key
    }

    pub(super) fn new(created: Instant, key: SessionKey, raw: spanner::Session) -> Self {
        Self {
            created,
            key,
            last_used: AtomicU64::new(0),
            state: AtomicU8::new(SESSION_ALIVE),
            raw,
        }
    }
}
