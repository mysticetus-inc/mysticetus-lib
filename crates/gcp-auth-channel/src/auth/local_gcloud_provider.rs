use std::ffi::OsString;
use std::fmt;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::Arc;

use gcp_auth::TokenProvider;
use tokio::sync::RwLock;

#[derive(Debug)]
pub(super) struct LocalGCloudProvider {
    cmd_path: PathBuf,
    state: RwLock<State>,
}

#[derive(Default)]
struct State {
    project_id: Option<Arc<str>>,
    fallback: Option<Arc<dyn TokenProvider>>,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("project_id", &self.project_id)
            .field("using_fallback", &self.fallback.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[repr(transparent)]
#[error("{0:?}")]
struct OsStringError(OsString);

impl LocalGCloudProvider {
    pub(super) fn try_load() -> std::io::Result<Option<Self>> {
        match find_gcloud()? {
            Some(cmd_path) => Ok(Some(Self {
                cmd_path,
                state: RwLock::new(State::default()),
            })),
            None => Ok(None),
        }
    }

    async fn get_or_insert_fallback_provider(
        &self,
    ) -> Result<Arc<dyn TokenProvider>, gcp_auth::Error> {
        let mut guard = self.state.write().await;

        match guard.fallback {
            Some(ref fallback) => Ok(fallback.clone()),
            None => {
                let fallback = gcp_auth::provider().await?;
                guard.fallback = Some(fallback.clone());
                Ok(fallback)
            }
        }
    }

    fn try_get_token_via_gcloud(&self) -> std::io::Result<gcp_auth::Token> {
        let output = std::process::Command::new(self.cmd_path.as_path())
            .arg("auth")
            .arg("print-access-token")
            .arg("--quiet") // not that this actually does anything
            .output()?;

        let token = get_relevant_gcloud_output(output)?;

        // the only way to create a gcp_auth::Token is via deserializing from json, since
        // they dont expose any methods to create one from a token + expiration time.
        let json_token_payload = format!("{{\"access_token\": \"{token}\", \"expires_in\": 3600}}");

        let token = serde_json::from_str(&json_token_payload)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        Ok(token)
    }

    fn try_get_project_id_via_gcloud(&self) -> std::io::Result<Arc<str>> {
        let output = std::process::Command::new(self.cmd_path.as_path())
            .arg("config")
            .arg("get-value")
            .arg("project")
            .output()?;

        let project_id = get_relevant_gcloud_output(output)?;

        Ok(Arc::from(project_id))
    }
}

fn get_relevant_gcloud_output(mut output: std::process::Output) -> std::io::Result<String> {
    // truncate after the first newline, since google likes to still
    // ask for survey feedback even with '--quiet'
    if let Some(idx) = memchr::memchr(b'\n', &output.stdout) {
        output.stdout.truncate(idx);
        output.stdout.shrink_to_fit();
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    if !output.status.success() || stdout.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            OsStringError(OsString::from_vec(output.stderr)),
        ));
    }

    Ok(stdout)
}

#[async_trait::async_trait]
impl TokenProvider for LocalGCloudProvider {
    async fn token(&self, scopes: &[&str]) -> Result<Arc<gcp_auth::Token>, gcp_auth::Error> {
        // if the fallback is set, we should use it
        if let Some(fallback_provider) = self.state.read().await.fallback.clone() {
            return fallback_provider.token(scopes).await;
        }

        match self.try_get_token_via_gcloud() {
            Ok(token) => Ok(Arc::new(token)),
            Err(error) => {
                eprintln!("failed to get token from gcloud: {error}, trying fallback");
                let fallback = self.get_or_insert_fallback_provider().await?;
                fallback.token(scopes).await
            }
        }
    }

    async fn project_id(&self) -> Result<Arc<str>, gcp_auth::Error> {
        if let Some(cached_project_id) = self.state.read().await.project_id.clone() {
            return Ok(cached_project_id);
        }

        let project_id = match self.try_get_project_id_via_gcloud() {
            Ok(project_id) => project_id,
            Err(error) => {
                eprintln!("failed to get project_id from gcloud: {error}, trying fallback");
                self.get_or_insert_fallback_provider()
                    .await?
                    .project_id()
                    .await?
            }
        };

        self.state.write().await.project_id = Some(project_id.clone());
        Ok(project_id)
    }
}

fn truncate_ascii_whitespace_end(buf: &mut Vec<u8>) {
    while buf.last().is_some_and(|byte| !byte.is_ascii_alphanumeric()) {
        buf.pop();
    }
}

fn find_gcloud() -> std::io::Result<Option<PathBuf>> {
    let mut output = std::process::Command::new("which")
        .arg("gcloud")
        .stdout(std::process::Stdio::piped())
        .output()?;

    // get rid of any trailing whitespace/ctrl chars, (there
    // shouldn't be any leading whitespace, so we only need
    // to look at the end)
    truncate_ascii_whitespace_end(&mut output.stdout);

    // 'which' returns an error code if the command wasnt found,
    // but also handle no stdout
    if !output.status.success() || output.stdout.is_empty() {
        return Ok(None);
    }

    Ok(Some(PathBuf::from(OsString::from_vec(output.stdout))))
}
