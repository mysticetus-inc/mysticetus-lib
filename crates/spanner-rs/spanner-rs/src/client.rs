use std::sync::Arc;

use gcp_auth_channel::{Auth, AuthChannel, Scope};
use protos::spanner::spanner_client::SpannerClient;
use shared::Shared;
use tonic::transport::ClientTlsConfig;

use crate::info::Database;
use crate::pool::SessionPool;
use crate::session::Session;

const DOMAIN: &str = "spanner.googleapis.com";
const ENDPOINT: &str = "https://spanner.googleapis.com";

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) info: Database,
    pub(crate) channel: AuthChannel,
    pub(crate) session_pool: Arc<SessionPool>,
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

    pub(crate) fn from_parts(database: Database, channel: AuthChannel) -> Self {
        Self {
            info: database,
            channel,
            session_pool: Arc::new(SessionPool::new()),
        }
    }

    #[cfg(all(feature = "emulator", feature = "admin"))]
    /// Sets up an emulator client, performing all DDL from the database this client points at.
    ///
    /// Returns the new emulator client + the running docker/emulator process
    pub async fn replicate_db_setup_emulator(
        &self,
        emulator_options: crate::emulator::EmulatorOptions,
        instance_compute: crate::admin::InstanceCompute,
    ) -> crate::Result<(crate::emulator::Emulator, Self)> {
        use anyhow::anyhow;
        use timestamp::Duration;

        let mut admin_client = self.admin_client();
        let db_info = self.info.clone();

        let load_ddl_task =
            tokio::spawn(async move { admin_client.get_database_ddl(&db_info).await });

        let emulator = crate::emulator::Emulator::start(emulator_options).await?;

        let ddl = load_ddl_task
            .await
            .map_err(|task_err| crate::Error::Misc(anyhow!(task_err)))??;

        let emulator_client = emulator
            .create_database(
                self.info.clone(),
                instance_compute,
                ddl,
                Some(Duration::from_seconds(30)),
            )
            .await?;

        Ok((emulator, emulator_client))
    }

    /// Create a new [`Client`] to interact with a different database. Clones
    /// the underlying gRPC channel, rather than expecting an entirely new client to
    /// be constructed
    pub fn with_database<S>(&self, database: Database<S>) -> Self
    where
        S: AsRef<str>,
        Shared<str>: From<S>,
    {
        Self {
            info: database.into_shared(),
            channel: self.channel.clone(),
            session_pool: Arc::clone(&self.session_pool),
        }
    }

    pub(crate) async fn new_load_auth<F, E>(info: Database, load_auth: F) -> crate::Result<Self>
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

        Ok(Self {
            channel,
            info,
            session_pool: Arc::new(SessionPool::new()),
        })
    }

    pub(crate) async fn new_loaded(info: Database, auth: Auth) -> crate::Result<Self> {
        let channel = build_channel().await?;

        let channel = AuthChannel::builder()
            .with_auth(auth)
            .with_channel(channel)
            .build();

        Ok(Self {
            channel,
            info,
            session_pool: Arc::new(SessionPool::new()),
        })
    }

    pub(crate) async fn new_inner(info: Database, scope: Scope) -> crate::Result<Self> {
        let project_id = info.project_id();
        let load_auth_fut = async move {
            gcp_auth_channel::Auth::new(project_id, scope)
                .await
                .map_err(crate::Error::from)
        };

        Self::new_load_auth(info, load_auth_fut).await
    }

    pub async fn new(
        project_id: &'static str,
        instance_name: &str,
        database_name: &str,
        scope: Scope,
    ) -> crate::Result<Self> {
        let info = Database::new(project_id, instance_name, database_name);

        Self::new_inner(info, scope).await
    }

    pub fn database_info(&self) -> &Database {
        &self.info
    }

    pub fn admin_client(&self) -> crate::admin::SpannerAdmin {
        crate::admin::SpannerAdmin::from_channel(self.channel.clone())
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

    pub(crate) fn grpc(&self) -> SpannerClient<AuthChannel> {
        SpannerClient::new(self.channel.clone())
    }

    async fn create_session_inner(&self, role: Option<String>) -> crate::Result<Session> {
        let req = protos::spanner::CreateSessionRequest {
            database: self.info.qualified_database().to_owned(),
            session: Some(protos::spanner::Session {
                creator_role: role.unwrap_or_else(String::new),
                ..Default::default()
            }),
        };

        let mut grpc = self.grpc();

        let session = grpc.create_session(req).await?.into_inner();

        Ok(Session::new(session, grpc))
    }

    pub async fn create_session(&self) -> crate::Result<Session> {
        self.create_session_inner(None).await
    }

    pub async fn create_session_with_role(
        &self,
        role: impl Into<String>,
    ) -> crate::Result<Session> {
        self.create_session_inner(Some(role.into())).await
    }
}
