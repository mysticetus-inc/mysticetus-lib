use gcp_auth_provider::scope::AccessLevel;
use gcp_auth_provider::{Auth, Scope};
use reqwest::header;
use shared::Shared;

use crate::Error;

/// A GCS client, scoped to a single bucket. Most methods only require a shared reference, at the
/// cost of them requiring an external [`String`] buffer for URL formatting. See [`StorageClient`]
/// for a version of this with an internal [`String`] buffer.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) client: reqwest::Client,
    pub(crate) auth: Auth,
}

impl Client {
    pub async fn new_admin() -> Result<Self, Error> {
        Self::new_with_scope(Scope::GcsAdmin).await
    }

    pub async fn new_read_only() -> Result<Self, Error> {
        Self::new_with_scope(Scope::GcsReadOnly).await
    }

    pub async fn new_with_scope(scope: Scope) -> Result<Self, Error> {
        let auth = Auth::new_detect().with_scopes(scope).await?;

        let client = reqwest::Client::new();
        Ok(Self { client, auth })
    }

    pub async fn new_with_access_level(access: AccessLevel) -> Result<Self, Error> {
        Self::new_with_scope(match access {
            AccessLevel::Admin => Scope::GcsAdmin,
            AccessLevel::ReadOnly => Scope::GcsReadOnly,
            AccessLevel::ReadWrite => Scope::GcsReadWrite,
        })
        .await
    }

    pub const fn into_bucket(self, bucket: Shared<str>) -> BucketClient {
        BucketClient {
            client: self,
            bucket,
            url_buffer: String::new(),
        }
    }

    pub fn from_parts(client: reqwest::Client, auth: Auth) -> Self {
        Self { client, auth }
    }

    pub fn bucket(&self, bucket: impl Into<Shared<str>>) -> BucketClient {
        self.clone().into_bucket(bucket.into())
    }

    pub fn auth(&self) -> &Auth {
        &self.auth
    }

    pub async fn delete(
        &self,
        url_buf: &mut String,
        bucket: &str,
        path: &str,
    ) -> Result<(), Error> {
        let auth_header = match self.auth.get_header() {
            gcp_auth_provider::GetHeaderResult::Cached(cache) => cache.header,
            gcp_auth_provider::GetHeaderResult::Refreshing(fut) => fut.await?.header,
        };

        crate::url::UrlBuilder::new(bucket)
            .name(path)
            .format_into(url_buf);

        let request = self
            .client
            .delete(url_buf.as_str())
            .header(header::AUTHORIZATION, auth_header)
            .build()?;

        crate::execute_and_validate_with_backoff(&self, request).await?;
        Ok(())
    }

    pub fn read<'a>(&'a self, bucket: &str, path: &str) -> crate::ReadBuilder<'a> {
        crate::ReadBuilder::new(self, bucket, path)
    }

    pub fn write<'a>(&'a self, bucket: &str, path: &'a str) -> crate::WriteBuilder<'a, ()> {
        crate::WriteBuilder::new(self, bucket, path)
    }

    pub fn list<'a>(&'a self, bucket: &str) -> crate::ListBuilder<'a> {
        crate::ListBuilder::new(self, bucket)
    }

    pub fn rewrite<'a>(
        &'a self,
        src_bucket: &'a str,
        src: &'a str,
    ) -> crate::rewrite::RewriteToBuilder<'a> {
        crate::rewrite::RewriteToBuilder::new(&self, src_bucket, src, None)
    }
}

/// A GCS client, scoped to a single bucket. Unlike [`SharedClient`], methods require
/// unique/mutable references, since it contains a [`String`] buffer internally.
#[derive(Debug, Clone)]
pub struct BucketClient {
    client: Client,
    bucket: Shared<str>,
    url_buffer: String,
}

impl BucketClient {
    pub fn new_bucket(&self, bucket: Shared<str>) -> Self {
        Self {
            client: self.client.clone(),
            bucket,
            url_buffer: String::new(),
        }
    }

    pub fn auth(&self) -> &Auth {
        &self.client.auth
    }

    pub fn from_parts(client: reqwest::Client, auth: Auth, bucket: Shared<str>) -> Self {
        Self {
            client: Client::from_parts(client, auth),
            bucket,
            url_buffer: String::new(),
        }
    }

    pub async fn new_with_scope<B>(scope: Scope, bucket_name: B) -> Result<Self, Error>
    where
        Shared<str>: From<B>,
    {
        Client::new_with_scope(scope)
            .await
            .map(|client| client.into_bucket(From::from(bucket_name)))
    }

    pub async fn delete(&mut self, path: &str) -> Result<(), Error> {
        self.client
            .delete(&mut self.url_buffer, &self.bucket, path)
            .await
    }

    pub async fn delete_opt(&mut self, path: &str) -> Result<bool, Error> {
        match self.delete(path).await {
            Ok(()) => Ok(true),
            Err(Error::NotFound(_)) => Ok(false),
            Err(other) => Err(other),
        }
    }

    pub fn read<'a>(&'a mut self, path: &str) -> crate::ReadBuilder<'a> {
        crate::ReadBuilder::new_buf(&self.client, &mut self.url_buffer, &self.bucket, path)
    }

    pub fn write<'a>(&'a mut self, path: &'a str) -> crate::WriteBuilder<'a, ()> {
        crate::WriteBuilder::new_buf(&self.client, &mut self.url_buffer, &self.bucket, path)
    }

    pub fn list<'a>(&'a mut self) -> crate::ListBuilder<'a> {
        crate::ListBuilder::new_buf(&self.client, &mut self.url_buffer, &self.bucket)
    }

    pub fn rewrite<'a>(&'a mut self, src: &'a str) -> crate::rewrite::RewriteToBuilder<'a> {
        crate::rewrite::RewriteToBuilder::new(
            &self.client,
            &self.bucket,
            src,
            Some(&mut self.url_buffer),
        )
    }

    pub async fn list_notification_configs(&self) -> Result<serde_json::Value, Error> {
        let header = match self.auth().get_header() {
            gcp_auth_provider::GetHeaderResult::Cached(cache) => cache.header,
            gcp_auth_provider::GetHeaderResult::Refreshing(fut) => fut.await?.header,
        };

        let url = format!(
            "https://storage.googleapis.com/storage/v1/b/{}/notificationConfigs",
            self.bucket
        );

        let request = self
            .client
            .client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, header)
            .build()?;

        let resp = crate::execute_and_validate_with_backoff(&self.client, request).await?;

        resp.json().await.map_err(Error::Reqwest)
    }
}

#[tokio::test]
async fn test_list_notifications() -> Result<(), Error> {
    let auth = gcp_auth_provider::Auth::new_detect()
        .with_scopes(Scope::GcsReadOnly)
        .await?;
    let client = Client::from_parts(Default::default(), auth)
        .into_bucket("mysticetus-replicated-data".into());

    let configs = client.list_notification_configs().await?;
    println!("{configs:#?}");
    Ok(())
}
