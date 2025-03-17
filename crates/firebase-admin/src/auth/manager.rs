use std::sync::Arc;

pub mod list_users;

#[derive(Debug, Clone)]
pub struct AuthManager {
    base_url: Arc<reqwest::Url>,
    auth: gcp_auth_channel::Auth,
    client: reqwest::Client,
}

impl AuthManager {
    pub async fn new(
        project_id: &'static str,
        scope: gcp_auth_channel::Scope,
    ) -> crate::Result<Self> {
        let auth = gcp_auth_channel::Auth::new(project_id, scope).await?;
        Ok(Self::from_auth(auth))
    }

    pub fn from_parts(auth: gcp_auth_channel::Auth, client: reqwest::Client) -> Self {
        let base_url = Arc::new(
            reqwest::Url::parse(&format!(
                "https://identitytoolkit.googleapis.com/v1/projects/{}",
                auth.project_id()
            ))
            .expect("invalid project id?"),
        );

        Self {
            base_url,
            auth,
            client,
        }
    }

    pub fn from_auth(auth: gcp_auth_channel::Auth) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("firebase-admin-rs")
            .build()
            .expect("valid client user agent");

        Self::from_parts(auth, client)
    }

    pub fn list_users(&self, page_size: usize) -> list_users::ListUsersStream {
        list_users::ListUsersStream::new(self.clone(), page_size)
    }

    async fn request(
        &self,
        url: impl reqwest::IntoUrl,
        method: reqwest::Method,
        build_request: impl FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
    ) -> crate::Result<reqwest::Response> {
        let auth_header = self.auth.get_header().await?;

        let builder = self
            .client
            .request(method, url)
            .header(reqwest::header::AUTHORIZATION, auth_header);

        build_request(builder)
            .send()
            .await
            .map_err(crate::Error::Reqwest)
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::OnceCell;

    use super::*;

    async fn get_manager() -> AuthManager {
        static MANAGER: OnceCell<AuthManager> = OnceCell::const_new();
        MANAGER
            .get_or_init(|| async {
                let auth = gcp_auth_channel::Auth::new(
                    "mysticetus-oncloud",
                    // std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
                    //     .expect("`$GOOGLE_APPLICATION_CREDENTIALS` is unset"),
                    gcp_auth_channel::Scope::CloudPlatformReadOnly,
                )
                .await
                .unwrap();
                println!("{auth:#?}");
                AuthManager::from_auth(auth)
            })
            .await
            .clone()
    }

    #[tokio::test]
    async fn test_list_users() -> crate::Result<()> {
        let manager = get_manager().await;

        let users = manager.list_users(20).collect().await?;
        println!("{users:#?}");

        Ok(())
    }
}
