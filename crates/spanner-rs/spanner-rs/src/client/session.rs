use std::sync::Arc;

use super::ClientParts;

pub struct SessionClient {
    parts: Arc<ClientParts>,
    session: super::pool::BorrowedSession,
}
