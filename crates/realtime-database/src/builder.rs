use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use gcp_auth_provider::{Auth, Scope};

use crate::client::Client;

pub enum DatabaseRef<'a> {
    Url(Cow<'a, str>),
    ProjectId,
}

impl DatabaseRef<'_> {
    fn to_url(self, project_id: &str) -> Arc<str> {
        match self {
            Self::Url(args) => Arc::from(args.to_string()),
            Self::ProjectId => Arc::from(format!("https://{project_id}.firebaseio.com")),
        }
    }
}

pub struct RealtimeDbBuilder<'a> {
    db_ref: DatabaseRef<'a>,
    auth_manager: Option<Auth>,
    silent_print: bool,
    use_cloud_platform_admin_scope: bool,
}

impl<'a> RealtimeDbBuilder<'a> {
    pub fn new() -> Self {
        Self {
            db_ref: DatabaseRef::ProjectId,
            auth_manager: None,
            silent_print: false,
            use_cloud_platform_admin_scope: false,
        }
    }

    pub fn database_url(mut self, database_url: impl Into<Cow<'a, str>>) -> Self {
        self.db_ref = DatabaseRef::Url(database_url.into());
        self
    }
}

impl RealtimeDbBuilder<'_> {
    pub fn enable_silent_print(mut self) -> Self {
        self.silent_print = true;
        self
    }

    pub fn with_auth_manager<A>(mut self, auth_manager: A) -> Self
    where
        A: Into<Auth>,
    {
        self.auth_manager = Some(auth_manager.into());
        self
    }
}

impl RealtimeDbBuilder<'_> {
    pub async fn from_service_account_file<P>(mut self, path: P) -> Result<Self, crate::Error>
    where
        P: Into<PathBuf>,
    {
        let scope = if self.use_cloud_platform_admin_scope {
            Scope::CloudPlatformAdmin
        } else {
            Scope::RealtimeDatabase
        };

        let auth_manager = Auth::from_service_account_file(path.into(), scope).await?;

        self.auth_manager = Some(auth_manager);
        Ok(self)
    }

    pub async fn build(self) -> Result<crate::RealtimeDatabase, crate::Error> {
        let http_client = reqwest::Client::builder()
            .user_agent("realtime-database-rs")
            .build()?;

        let auth = match self.auth_manager {
            Some(auth) => auth,
            None => {
                Auth::new_detect()
                    .with_scopes(if self.use_cloud_platform_admin_scope {
                        Scope::CloudPlatformAdmin
                    } else {
                        Scope::RealtimeDatabase
                    })
                    .await?
            }
        };
        let db_url = self.db_ref.to_url(&auth.project_id());

        let client = Client::new(db_url, auth, http_client, self.silent_print);

        Ok(crate::RealtimeDatabase { client })
    }
}
