use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};

use gcp_auth_channel::AuthChannel;
use net_utils::backoff::Backoff;
use protos::spanner;
use protos::spanner::spanner_client::SpannerClient;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tonic::Code;

use self::queue::{BorrowQueue, Borrowed};
use crate::Client;

const MAX_SESSIONS: usize = 100;

static POOL_DEBUG: LazyLock<bool> = LazyLock::new(|| std::env::var("POOL_DEBUG").is_ok());

pub struct Session<'a> {
    // expected to be Some until Drop
    session: Option<Borrowed<'a, WrappedSession>>,
    // hold onto the a clone of the session name separately, that way we can access it without
    // having to unwrap the Option
    session_name: Arc<str>,
    client: Client,
}

impl fmt::Debug for Session<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.session.as_ref() {
            Some(inner) => inner.session.fmt(f),
            None => f
                .debug_struct("Session")
                .field("name", &&*self.session_name)
                .finish_non_exhaustive(),
        }
    }
}

impl Session<'_> {
    pub fn name(&self) -> &str {
        &self.session_name
    }

    pub fn name_arc(&self) -> &Arc<str> {
        &self.session_name
    }

    // let Drop do the work
    pub fn finish(self) {}
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            // if we couldn't return the session, try and delete it instead
            if let Err(session) = session.return_to_queue() {
                let client = self.client.grpc();
                spawn_delete_session(client, session.session);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("couldn't create {requested} sessions, {total} live sessions")]
    CantCreateSession { requested: usize, total: usize },
    #[error("pool full, {in_use}/{total} sessions in use")]
    PoolFull { in_use: usize, total: usize },
    #[error("couldn't create sessions ({requested} requested): {response:?}")]
    NoSessionsCreated {
        requested: usize,
        response: tonic::Response<spanner::BatchCreateSessionsResponse>,
    },
    #[error("timed out while waiting for a session to become available")]
    TimedOutWaitingForSession,
}

pub struct SessionPool {
    sessions: BorrowQueue<WrappedSession>,
    batch_create_size: AtomicUsize,
    notify_new: Notify,
}

impl fmt::Debug for SessionPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionPool")
            .field("sessions", &self.sessions.capacity())
            .field("sessions_available", &self.sessions.available())
            .field("batch_create_size", &self.batch_create_size)
            .field("stats", &self.stats())
            .finish_non_exhaustive()
    }
}

/// Used to keep a cached Arc<str> with the sessions name. That way we don't
/// need to re-create an Arc<str> every time a session is checked out
struct WrappedSession {
    name: Arc<str>,
    session: spanner::Session,
}

impl WrappedSession {
    fn new(session: spanner::Session) -> Self {
        Self {
            name: Arc::from(session.name.as_str()),
            session,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPoolStats {
    pub capacity: usize,
    pub total: usize,
    pub available: usize,
    pub in_use: usize,
}

impl SessionPool {
    pub fn new() -> Self {
        Self {
            sessions: BorrowQueue::new(MAX_SESSIONS),
            batch_create_size: AtomicUsize::new(5),
            notify_new: Notify::new(),
        }
    }

    pub async fn try_load_existing_sessions(
        &self,
        client: &Client,
    ) -> crate::Result<Option<usize>> {
        if self.sessions.total() > 0 {
            return Ok(None);
        }

        let mut scoped = client.grpc();

        let req = spanner::ListSessionsRequest {
            database: client.info.qualified_database().to_owned(),
            page_size: 100,
            page_token: String::new(),
            filter: String::new(),
        };

        let sessions = scoped.list_sessions(req).await?.into_inner();

        let mut added = 0;
        for session in sessions.sessions {
            if self.sessions.push_new(WrappedSession::new(session)).is_ok() {
                added += 1;
            }
        }

        Ok(Some(added))
    }

    pub fn stats(&self) -> SessionPoolStats {
        let (in_use, total) = self.sessions.in_use_and_total();
        SessionPoolStats {
            capacity: self.sessions.capacity(),
            available: self.sessions.available(),
            total,
            in_use,
        }
    }

    pub fn set_batch_create_size(&self, b: usize) {
        assert_ne!(b, 0);
        self.batch_create_size.store(b, Ordering::SeqCst);
    }

    pub fn get_session(&self, client: &Client) -> Option<Session<'_>> {
        self.sessions.borrow().map(|session| Session {
            session_name: Arc::clone(&session.name),
            session: Some(session),
            client: client.clone(),
        })
    }

    pub async fn get_existing_session<'a>(
        &'a self,
        client: &Client,
    ) -> crate::Result<Option<Session<'a>>> {
        // lazily create a scoped client, since it requires cloning a couple Arc's and a string (for
        // the Database).
        let mut scoped = None;

        loop {
            let Some(mut session) = self.sessions.borrow() else {
                return Ok(None);
            };

            let scoped = scoped.get_or_insert_with(|| client.grpc());

            let req = spanner::GetSessionRequest {
                name: session.name.as_ref().to_owned(),
            };

            match scoped.get_session(req).await {
                Ok(resp) => {
                    session.update_session(resp.into_inner());
                    return Ok(Some(Session {
                        session_name: Arc::clone(&session.name),
                        session: Some(session),
                        client: client.clone(),
                    }));
                }
                Err(status) if status.code() == Code::NotFound => continue,
                Err(status) => return Err(crate::Error::Status(status)),
            }
        }
    }

    pub async fn wait_for_session<'a>(
        &'a self,
        client: &Client,
        timeout: timestamp::Duration,
    ) -> Option<Session<'a>> {
        let session = match self.sessions.borrow_or_wait_with_timeout(timeout) {
            Ok(session) => session,
            Err(fut) => match fut.await {
                Ok(session) => session,
                Err(_) => return None,
            },
        };

        Some(Session {
            session_name: Arc::clone(&session.name),
            session: Some(session),
            client: client.clone(),
        })
    }

    pub async fn get_or_create_session<'a>(
        &'a self,
        client: &Client,
    ) -> crate::Result<Session<'a>> {
        if let Some(sess) = self.get_session(client) {
            return Ok(sess);
        }

        self.batch_create_sessions(self.batch_create_size.load(Ordering::SeqCst), client, None)
            .await?;

        self.wait_for_session(client, timestamp::Duration::from_seconds(5))
            .await
            .ok_or_else(|| SessionError::TimedOutWaitingForSession.into())
    }

    pub async fn batch_create_sessions(
        &self,
        n: usize,
        client: &Client,
        role: Option<String>,
    ) -> crate::Result<()> {
        assert_ne!(n, 0);

        let n = self.sessions.space_remaining().min(n);

        if n == 0 {
            let (in_use, total) = self.sessions.in_use_and_total();
            return Err(SessionError::PoolFull { in_use, total }.into());
        }

        let req = spanner::BatchCreateSessionsRequest {
            database: client.info.qualified_database().to_owned(),
            session_count: n as i32,
            session_template: role.map(|creator_role| spanner::Session {
                creator_role,
                ..Default::default()
            }),
        };

        let response = client.grpc().batch_create_sessions(req).await?;

        if response.get_ref().session.is_empty() {
            return Err(SessionError::NoSessionsCreated {
                requested: n,
                response,
            }
            .into());
        }

        if *POOL_DEBUG {
            info!(
                message = "created new sessions",
                number = response.get_ref().session.len()
            );
        }

        for sess in response.into_inner().session {
            if let Some(old) = self.sessions.force_push_new(WrappedSession::new(sess)) {
                let client = client.grpc();
                tokio::spawn(async move {
                    let _ = delete_session(client, old.session).await;
                });
            } else {
                self.notify_new.notify_one();
            }
        }

        Ok(())
    }

    pub fn delete_unused(
        &self,
        client: &Client,
    ) -> tokio::task::JoinSet<Result<(), (spanner::Session, tonic::Status)>> {
        let mut join_set = tokio::task::JoinSet::new();

        while let Some(sess) = self.sessions.remove() {
            let client = client.grpc();
            join_set.spawn(async move { delete_session(client, sess.session).await });
        }

        join_set
    }

    pub async fn on_shutdown(&self, client: &Client) {
        if *POOL_DEBUG {
            info!(message="spanner session pool shutdown starting...", stats=?self.stats());
        }

        let mut join_set = self.delete_unused(client);

        loop {
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(())) => {
                        if *POOL_DEBUG {
                            info!(
                                "deleted spanner session, {} remaining",
                                self.sessions.total() + join_set.len()
                            );
                        }
                    }
                    Ok(Err((sess, status))) => error!(
                        message = "couldn't delete spanner session",
                        ?status,
                        name = sess.name
                    ),
                    Err(error) => error!(
                        message = "panic'd while trying to delete spanner session",
                        ?error
                    ),
                }
            }

            // if all sessions have been removed, we can bail
            if self.sessions.total() == 0 {
                break;
            }

            // otherwise, try and pull out more that got missed when calling 'delete_unused'
            if let Some(sess) = self.sessions.remove() {
                let client = client.grpc();
                join_set.spawn(delete_session(client, sess.session));
            }
        }

        if *POOL_DEBUG {
            info!(message="spanner session pool shutdown complete", stats=?self.stats());
        }
    }
}

fn spawn_delete_session(
    client: SpannerClient<AuthChannel>,
    session: spanner::Session,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err((session, error)) = delete_session(client, session).await {
            error!(
                message = "error deleting session",
                ?error,
                name = session.name
            );
        }
    })
}

async fn delete_session(
    mut client: SpannerClient<AuthChannel>,
    session: spanner::Session,
) -> Result<(), (spanner::Session, tonic::Status)> {
    let mut backoff = Backoff::default();

    loop {
        let status = match client
            .delete_session(spanner::DeleteSessionRequest {
                name: session.name.clone(),
            })
            .await
        {
            Ok(_) => return Ok(()),
            // if spanner deletes a session underneath us, dont error out since we're trying to do
            // just that
            Err(status) if status.code() == Code::NotFound => return Ok(()),
            Err(status) => status,
        };

        match backoff.backoff_once() {
            Some(backoff) => {
                warn!(message = "error deleting session", ?status);
                backoff.await;
            }
            None => return Err((session, status)),
        }
    }
}

pub(crate) mod queue {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll};

    use crossbeam::queue::ArrayQueue;
    use tokio::sync::futures::Notified;
    use tokio::sync::Notify;

    pub(crate) struct Borrowed<'a, T> {
        item: T,
        queue: &'a BorrowQueue<T>,
    }

    impl Borrowed<'_, super::WrappedSession> {
        pub(super) fn update_session(&mut self, sess: protos::spanner::Session) {
            self.item.session = sess;
        }
    }

    impl<T> Borrowed<'_, T> {
        pub fn remove(self) -> T {
            self.queue.total.fetch_sub(1, Ordering::SeqCst);
            self.item
        }

        pub fn return_to_queue(self) -> Result<(), T> {
            self.queue.queue.push(self.item)
        }

        pub fn force_return_to_queue(self) -> Option<T> {
            self.queue.queue.force_push(self.item)
        }
    }

    impl<T> std::ops::Deref for Borrowed<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.item
        }
    }

    pub(crate) struct BorrowQueue<T> {
        queue: ArrayQueue<T>,
        total: AtomicUsize,
        notify_new: Notify,
    }

    pub(crate) type WaitForBorrowedTimeout<'a, T> = tokio::time::Timeout<WaitForBorrowed<'a, T>>;

    impl<T> BorrowQueue<T> {
        pub fn new(cap: usize) -> Self {
            Self {
                queue: ArrayQueue::new(cap),
                total: AtomicUsize::new(0),
                notify_new: Notify::new(),
            }
        }

        pub fn space_remaining(&self) -> usize {
            self.queue.capacity().saturating_sub(self.queue.len())
        }

        pub fn in_use_and_total(&self) -> (usize, usize) {
            let total = self.total.load(Ordering::SeqCst);
            (total.saturating_sub(self.queue.len()), total)
        }

        pub fn in_use(&self) -> usize {
            self.total
                .load(Ordering::SeqCst)
                .saturating_sub(self.queue.len())
        }

        pub fn capacity(&self) -> usize {
            self.queue.capacity()
        }

        pub fn total(&self) -> usize {
            self.total.load(Ordering::SeqCst)
        }

        pub fn available(&self) -> usize {
            self.queue.len()
        }

        pub fn borrow(&self) -> Option<Borrowed<'_, T>> {
            self.queue.pop().map(|item| Borrowed { item, queue: self })
        }

        pub fn borrow_or_wait(&self) -> Result<Borrowed<'_, T>, WaitForBorrowed<'_, T>> {
            self.borrow().ok_or_else(|| WaitForBorrowed {
                notified: self.notify_new.notified(),
                queue: self,
            })
        }

        pub fn borrow_or_wait_with_timeout(
            &self,
            timeout: timestamp::Duration,
        ) -> Result<Borrowed<'_, T>, WaitForBorrowedTimeout<'_, T>> {
            self.borrow().ok_or_else(|| {
                tokio::time::timeout(
                    timeout.into(),
                    WaitForBorrowed {
                        notified: self.notify_new.notified(),
                        queue: self,
                    },
                )
            })
        }

        pub fn push_new(&self, item: T) -> Result<(), T> {
            self.queue.push(item)?;
            self.total.fetch_add(1, Ordering::SeqCst);
            self.notify_new.notify_one();
            Ok(())
        }

        pub fn force_push_new(&self, item: T) -> Option<T> {
            match self.queue.force_push(item) {
                Some(old) => Some(old),
                None => {
                    self.notify_new.notify_one();
                    self.total.fetch_add(1, Ordering::SeqCst);
                    None
                }
            }
        }

        pub fn remove(&self) -> Option<T> {
            if let Some(item) = self.queue.pop() {
                self.total.fetch_sub(1, Ordering::SeqCst);
                Some(item)
            } else {
                None
            }
        }
    }

    pin_project_lite::pin_project! {
        pub(crate) struct WaitForBorrowed<'a, T> {
            #[pin]
            notified: Notified<'a>,
            queue: &'a BorrowQueue<T>,
        }
    }

    impl<'a, T> Future for WaitForBorrowed<'a, T> {
        type Output = Borrowed<'a, T>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut this = self.project();

            loop {
                std::task::ready!(this.notified.as_mut().poll(cx));

                if let Some(borrow) = this.queue.borrow() {
                    return Poll::Ready(borrow);
                }

                // otherwise, reset the notification and loop again so the waker
                // can be registered.
                this.notified.set(this.queue.notify_new.notified());
            }
        }
    }
}
