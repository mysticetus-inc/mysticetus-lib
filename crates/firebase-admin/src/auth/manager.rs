use std::sync::Arc;

pub mod oob;
pub use oob::{OobRequest, OobRequestType, OobResponse};

pub mod list_users;

const BASE_URL: &str = "https://identitytoolkit.googleapis.com/v1";
const OOB_URL: &str = "https://identitytoolkit.googleapis.com/v1/accounts:sendOobCode";

#[derive(Debug, Clone)]
pub struct AuthManager {
    base_url: Arc<reqwest::Url>,
    auth: gcp_auth_provider::Auth,
    client: reqwest::Client,
}

impl AuthManager {
    pub async fn new(scope: gcp_auth_provider::Scope) -> crate::Result<Self> {
        let auth = gcp_auth_provider::Auth::new_detect()
            .with_scopes(scope)
            .await?;

        Ok(Self::from_auth(auth))
    }

    pub fn from_parts(auth: gcp_auth_provider::Auth, client: reqwest::Client) -> Self {
        let base_url = Arc::new(
            reqwest::Url::parse(&format!("{BASE_URL}/projects/{}", auth.project_id()))
                .expect("invalid project id?"),
        );

        Self {
            base_url,
            auth,
            client,
        }
    }

    pub fn from_auth(auth: gcp_auth_provider::Auth) -> Self {
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
        let auth_header = match self.auth.get_header() {
            gcp_auth_provider::GetHeaderResult::Cached(cached) => cached.header,
            gcp_auth_provider::GetHeaderResult::Refreshing(fut) => fut.await?.header,
        };

        let builder = self
            .client
            .request(method, url)
            .header("x-goog-user-project", self.auth.project_id())
            .header(reqwest::header::AUTHORIZATION, auth_header);

        build_request(builder)
            .send()
            .await
            .map_err(crate::Error::Reqwest)
    }

    pub async fn send_oob_code<T: OobRequestType>(
        &self,
        request: &OobRequest<'_, T>,
    ) -> crate::Result<OobResponse> {
        let response = self
            .request(OOB_URL, reqwest::Method::POST, |builder| {
                builder.json(request)
            })
            .await?;

        parse_json_response(response).await
    }
}

async fn parse_json_response<T>(response: reqwest::Response) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    let uri = response.url().clone();
    let bytes = response.bytes().await?;

    // handle the successful case first
    if status.is_success() {
        let value = path_aware_serde::json::deserialize_slice(&bytes)?;
        return Ok(value);
    }

    Err(crate::Error::Status(crate::error::StatusError::new_from(
        uri, status, bytes,
    )))
}

#[cfg(test)]
mod tests {
    use tokio::sync::OnceCell;

    use super::*;

    async fn get_manager() -> AuthManager {
        static MANAGER: OnceCell<AuthManager> = OnceCell::const_new();
        MANAGER
            .get_or_init(|| async {
                let auth = gcp_auth_provider::Auth::new_detect()
                    .with_scopes(gcp_auth_provider::Scope::CloudPlatformAdmin)
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

    #[tokio::test]
    async fn test_generate_email_sign_in_link() -> crate::Result<()> {
        let manager = get_manager().await;

        let req = OobRequest::new("mrudisel@mysticetus.com", "http::localhost:3000");

        match manager.send_oob_code::<oob::EmailSignIn>(&req).await {
            Err(error) => {
                println!("{error:#?}");
                Err(error)
            }
            Ok(codes) => {
                println!("{codes:#?}");
                Ok(())
            }
        }
    }
}
