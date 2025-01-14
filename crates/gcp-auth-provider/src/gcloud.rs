use std::ffi::OsString;
use std::future::Future;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct GCloudProvider {
    cmd_path: Box<Path>,
}

impl GCloudProvider {
    pub fn new(cmd_path: impl Into<PathBuf>) -> Self {
        Self {
            cmd_path: PathBuf::into_boxed_path(cmd_path.into()),
        }
    }
}

impl super::RawTokenProvider for GCloudProvider {
    const NAME: &'static str = "gcloud";

    fn try_load() -> impl Future<Output = crate::Result<Option<Self>>> + Send + 'static {
        async move {
            match find_gcloud().await? {
                Some(cmd_path) => Ok(Some(Self::new(cmd_path))),
                None => Ok(None),
            }
        }
    }

    fn get_token(&self) -> impl Future<Output = crate::Result<crate::Token>> + Send + 'static {
        let child_res = tokio::process::Command::new(&*self.cmd_path)
            .arg("auth")
            .arg("print-access-token")
            .arg("--quiet") // not that this actually does anything
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();

        async move {
            let output = child_res?.wait_with_output().await?;

            let access_token = get_relevant_gcloud_output(output)?.into_boxed_str();

            Ok(crate::Token::new_with_default_expiry_time(access_token))
        }
    }

    fn project_id(&self) -> impl Future<Output = crate::Result<Arc<str>>> + Send + 'static {
        let child_res = tokio::process::Command::new(&*self.cmd_path)
            .arg("config")
            .arg("get-value")
            .arg("project")
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();

        async move {
            let output = child_res?.wait_with_output().await?;

            let project_id = get_relevant_gcloud_output(output)?;

            Ok(Arc::from(project_id))
        }
    }
}

async fn find_gcloud() -> std::io::Result<Option<PathBuf>> {
    fn truncate_ascii_whitespace_end(buf: &mut Vec<u8>) {
        while buf.last().is_some_and(|byte| !byte.is_ascii_alphanumeric()) {
            buf.pop();
        }
    }

    let mut output = tokio::process::Command::new("which")
        .arg("gcloud")
        .output()
        .await?;

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

fn get_relevant_gcloud_output(mut output: std::process::Output) -> std::io::Result<String> {
    #[derive(Debug, Clone, thiserror::Error)]
    #[repr(transparent)]
    #[error("{0:?}")]
    struct OsStringError(OsString);

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
