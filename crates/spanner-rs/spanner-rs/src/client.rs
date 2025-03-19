use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use gcp_auth_channel::{Auth, AuthChannel, Scope};
use protos::spanner::spanner_client::SpannerClient;
use timestamp::Timestamp;
use tokio::task::JoinSet;
use tonic::transport::ClientTlsConfig;

use crate::info::Database;
use crate::key_set::WriteBuilder;
use crate::tx::{ShouldCommit, Transaction};
use crate::{ResultIter, StreamingRead, Table};

pub mod pool;

#[cfg(feature = "admin")]
pub mod admin;
#[cfg(feature = "emulator")]
pub mod emulator;

mod session;

pub(crate) mod connection;

use pool::SessionPool;
pub use session::SessionClient;

const DOMAIN: &str = "spanner.googleapis.com";
const ENDPOINT: &str = "https://spanner.googleapis.com";

#[derive(Clone)]
pub struct Client {
    parts: Arc<ClientParts>,
    session_pool: SessionPool,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("info", &self.parts.info)
            .field("channel", &self.parts.channel)
            .field("session_pool", &self.session_pool)
            .finish()
    }
}

pub(crate) struct ClientParts {
    pub(crate) info: Database,
    pub(crate) channel: AuthChannel,
    pub(crate) role: Option<Box<str>>,
}

async fn build_channel() -> crate::Result<tonic::transport::Channel> {
    tonic::transport::Channel::from_static(ENDPOINT)
        .user_agent(gcp_auth_channel::user_agent!())?
        .tls_config(
            ClientTlsConfig::new()
                .domain_name(DOMAIN)
                .with_webpki_roots(),
        )?
        .connect()
        .await
        .map_err(crate::Error::from)
}

impl Client {
    /// Shortcut call to [`Database::builder`].
    pub fn builder(project_id: &'static str) -> crate::info::Project {
        Database::builder(project_id)
    }

    pub(crate) fn from_parts(info: Database, channel: AuthChannel, role: Option<Box<str>>) -> Self {
        let parts = Arc::new(ClientParts {
            info,
            channel,
            role,
        });

        Self {
            session_pool: SessionPool::new(Arc::clone(&parts)),
            parts,
        }
    }

    #[cfg(all(feature = "emulator", feature = "admin"))]
    /// Sets up an emulator client, performing all DDL from the database this client points at.
    ///
    /// Returns the new emulator client + the running docker/emulator process
    pub async fn replicate_db_setup_emulator(
        &self,
        emulator_options: emulator::EmulatorOptions,
        instance_compute: admin::InstanceCompute,
    ) -> crate::Result<(emulator::Emulator, Self)> {
        use anyhow::anyhow;
        use timestamp::Duration;

        let mut admin_client = self.admin_client();
        let db_info = self.parts.info.clone();

        let load_ddl_task =
            tokio::spawn(async move { admin_client.get_database_ddl(&db_info).await });

        let emulator = emulator::Emulator::start(emulator_options).await?;

        let ddl = load_ddl_task
            .await
            .map_err(|task_err| crate::Error::Misc(anyhow!(task_err)))??;

        let emulator_client = emulator
            .create_database(
                self.parts.info.clone(),
                instance_compute,
                ddl,
                Some(Duration::from_seconds(30)),
            )
            .await?;

        Ok((emulator, emulator_client))
    }

    pub(crate) async fn new_load_auth<F, E>(
        info: Database,
        role: Option<Box<str>>,
        load_auth: F,
    ) -> crate::Result<Self>
    where
        F: std::future::Future<Output = Result<Auth, E>>,
        E: Into<crate::Error>,
    {
        use futures::TryFutureExt;
        let load_auth_map = load_auth.map_err(Into::into);
        futures::pin_mut!(load_auth_map);

        let (channel, auth) = tokio::try_join!(build_channel(), load_auth_map)?;

        let channel = AuthChannel::builder()
            .with_auth(auth)
            .with_channel(channel)
            .build();

        Ok(Self::from_parts(info, channel, role))
    }

    pub(crate) async fn new_loaded(
        info: Database,
        auth: Auth,
        role: Option<Box<str>>,
    ) -> crate::Result<Self> {
        let channel = build_channel().await?;

        let channel = AuthChannel::builder()
            .with_auth(auth)
            .with_channel(channel)
            .build();

        Ok(Self::from_parts(info, channel, role))
    }

    pub(crate) async fn new_inner(
        info: Database,
        scope: Scope,
        role: Option<Box<str>>,
    ) -> crate::Result<Self> {
        let project_id = info.project_id();
        let load_auth_fut = async move {
            gcp_auth_channel::Auth::new(project_id, scope)
                .await
                .map_err(crate::Error::from)
        };

        Self::new_load_auth(info, role, load_auth_fut).await
    }

    pub async fn new(
        project_id: &'static str,
        instance_name: &str,
        database_name: &str,
        scope: Scope,
        role: Option<Box<str>>,
    ) -> crate::Result<Self> {
        let info = Database::new(project_id, instance_name, database_name);

        Self::new_inner(info, scope, role).await
    }

    pub(crate) fn parts(&self) -> &Arc<ClientParts> {
        &self.parts
    }

    pub fn database_info(&self) -> &Database {
        &self.parts.info
    }

    pub(crate) fn channel(&self) -> &AuthChannel {
        &self.parts.channel
    }

    #[cfg(feature = "admin")]
    pub fn admin_client(&self) -> admin::SpannerAdmin {
        admin::SpannerAdmin::from_channel(self.parts.channel.clone())
    }

    /*
    async fn batch_create_sessions_inner<const N: usize>(
        &mut self,
    ) -> crate::Result<> {
        let req = protos::spanner::BatchCreateSessionsRequest {
            database: self.info.qualified_database().to_owned(),
            session_template: None,
            session_count: N as i32,
        };
        let resp = self.client().batch_create_sessions(req).await?.into_inner();

        assert!(resp.session.len() <= N, "more sessions than requested?");

        let sessions = resp
            .session
            .into_iter()
            .map(|session| Session::new(session, self))
            .collect::<Stack<N, Session>>();

        Ok(sessions)
    }

    pub async fn batch_create_sessions<const N: usize>(
        &self,
    ) -> crate::Result<Stack<N, Session>> {
        let req = protos::spanner::BatchCreateSessionsRequest {
            database: self.info.qualified_database().to_owned(),
            session_template: None,
            session_count: N as i32,
        };
        let resp = self.client().batch_create_sessions(req).await?.into_inner();

        assert!(resp.session.len() <= N, "more sessions than requested?");

        let sessions = resp
            .session
            .into_iter()
            .map(|session| Session::new(session, self))
            .collect::<Stack<N, Session>>();

        Ok(sessions)
    }
    */

    pub fn shutdown_task(&self) -> impl FnOnce() -> ShutdownFuture {
        let pool = self.session_pool.clone();

        move || ShutdownFuture {
            tasks: pool.delete_sessions(),
        }
    }

    pub async fn borrow_session(
        &self,
        timeout: Option<timestamp::Duration>,
    ) -> crate::Result<SessionClient> {
        let session = self
            .session_pool
            .get_or_create_session_according_to_load(timeout)
            .await?;

        Ok(SessionClient {
            parts: self.parts.clone(),
            session,
        })
    }

    pub fn session_role(&self) -> Option<&str> {
        self.parts.role.as_deref()
    }

    // crate::connection::impl_deferred_read_functions!();

    pub async fn insert_or_update<T: crate::Table>(
        &self,
        rows: WriteBuilder<T>,
    ) -> crate::Result<Option<protos::spanner::commit_response::CommitStats>> {
        self.borrow_session(None)
            .await?
            .insert_or_update(rows)
            .await
    }

    pub async fn run_in_transaction<F, Fut>(&self, func: F) -> crate::Result<Option<Timestamp>>
    where
        F: FnMut(&mut Transaction<'_, '_>) -> Fut,
        Fut: std::future::Future<Output = crate::Result<ShouldCommit>>,
    {
        self.borrow_session(None)
            .await?
            .run_in_transaction(func)
            .await
    }

    pub(crate) async fn execute_dml(
        &self,
        statements: Vec<protos::spanner::execute_batch_dml_request::Statement>,
    ) -> crate::Result<protos::spanner::ExecuteBatchDmlResponse> {
        self.borrow_session(None)
            .await?
            .execute_dml(statements)
            .await
    }

    pub async fn execute_streaming_sql<T: Table>(
        &mut self,
        sql: String,
        params: Option<crate::sql::Params>,
    ) -> crate::Result<StreamingRead<T>> {
        self.borrow_session(None)
            .await?
            .execute_streaming_sql(sql, params)
            .await
    }

    pub async fn execute_sql<T: Table>(
        &mut self,
        sql: String,
        params: Option<crate::sql::Params>,
    ) -> crate::Result<ResultIter<T>> {
        self.borrow_session(None)
            .await?
            .execute_sql(sql, params)
            .await
    }
}

pin_project_lite::pin_project! {
    pub struct ShutdownFuture {
        #[pin]
        tasks: Option<JoinSet<crate::Result<()>>>,
    }
}

impl std::future::Future for ShutdownFuture {
    type Output = crate::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let Some(mut tasks) = this.tasks.as_mut().as_pin_mut() else {
            return Poll::Ready(Ok(()));
        };

        loop {
            match std::task::ready!(tasks.poll_join_next(cx)) {
                Some(result) => {
                    result.map_err(|task_err| crate::Error::Misc(task_err.into()))??
                }
                None => {
                    this.tasks.set(None);
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}
