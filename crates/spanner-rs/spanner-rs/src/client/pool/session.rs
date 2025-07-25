use std::hash::BuildHasher;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

use gcp_auth_channel::AuthChannel;
use protos::spanner::spanner_client::SpannerClient;
use protos::spanner::{self, DeleteSessionRequest};

use crate::client::ClientParts;
use crate::client::pool::tracker::SessionKey;

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
            let mut channel = parts.channel.clone();

            match self.delete_inner(&mut channel).await {
                Ok(()) => return Ok(()),
                Err(err) if err.code() == tonic::Code::Unauthenticated => {
                    // force refresh the token if we get an auth error, then try again once
                    parts.channel.auth().revoke_token(true);
                    self.delete_inner(&mut channel).await?;
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(())
    }

    async fn delete_inner(&self, channel: &mut AuthChannel) -> Result<(), tonic::Status> {
        SpannerClient::new(channel)
            .delete_session(DeleteSessionRequest {
                name: self.raw.name.clone(),
            })
            .await?;

        self.state.store(SESSION_DELETED, Ordering::Release);

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
