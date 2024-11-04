#![allow(dead_code)]
use gcp_auth_channel::{Auth, Scope};

use crate::Error;

const ADMIN_SCOPES: &[Scope] = &[Scope::CloudPlatformAdmin, Scope::GcsAdmin];

#[derive(Debug)]
pub struct BucketClient {
    client: reqwest::Client,
    auth: Auth,
}

impl BucketClient {
    pub async fn new(project_id: &'static str, scope: Scope) -> Result<Self, Error> {
        let auth_manager = Auth::new(project_id, scope).await?;
        let client = reqwest::ClientBuilder::new().build()?;

        Ok(Self::from_parts(client, auth_manager))
    }

    pub fn from_parts(client: reqwest::Client, auth: Auth) -> Self {
        Self { client, auth }
    }
}
