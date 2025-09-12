use std::ffi::OsString;
use std::future::Future;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::{Error, ProjectId};

#[derive(Debug, Clone)]
pub struct GCloudProvider {
    cmd_path: Arc<Path>,
}

impl GCloudProvider {
    pub fn new(cmd_path: impl Into<PathBuf>) -> Self {
        Self {
            cmd_path: Arc::from(PathBuf::into_boxed_path(cmd_path.into())),
        }
    }
}

// since this shells out to a new process, blocking impls are actually preferred
// so run them in a task.
pub struct GCloudFuture<T>(tokio::task::JoinHandle<crate::Result<T>>);

impl<T> Future for GCloudFuture<T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io::{self, ErrorKind};

        match std::task::ready!(Pin::new(&mut self.get_mut().0).poll(cx)) {
            Ok(result) => Poll::Ready(result),
            Err(err) if err.is_cancelled() => {
                Poll::Ready(Err(Error::Io(io::Error::from(ErrorKind::Interrupted))))
            }
            Err(err) => Poll::Ready(Err(Error::Io(io::Error::new(ErrorKind::Other, err)))),
        }
    }
}

impl GCloudProvider {
    pub fn try_load() -> GCloudFuture<Option<(Self, ProjectId)>> {
        GCloudFuture(tokio::task::spawn_blocking(|| {
            let cmd_path = match find_gcloud()? {
                Some(cmd_path) => cmd_path,
                None => return Ok(None),
            };

            let output = std::process::Command::new(&cmd_path)
                .arg("config")
                .arg("get-value")
                .arg("project")
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()?;

            let project_id = ProjectId::new_shared_owned(get_relevant_gcloud_output(output)?);

            let prov = Self {
                cmd_path: Arc::from(cmd_path),
            };

            Ok(Some((prov, project_id)))
        }))
    }
}

impl super::BaseTokenProvider for GCloudProvider {
    #[inline]
    fn name(&self) -> &'static str {
        "gcloud"
    }
}

impl super::TokenProvider for GCloudProvider {
    fn get_token(&self) -> crate::GetTokenFuture<'_> {
        let mut cmd = std::process::Command::new(&*self.cmd_path);

        crate::GetTokenFuture::new_gcloud(GCloudFuture(tokio::task::spawn_blocking(move || {
            let output = cmd
                .arg("auth")
                .arg("print-access-token")
                .arg("--quiet") // not that this actually does anything
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()?;

            let access_token = get_relevant_gcloud_output(output)?;

            crate::Token::new_with_default_expiry_time(&access_token).map_err(|err| {
                crate::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
            })
        })))
    }
}

fn find_gcloud() -> std::io::Result<Option<PathBuf>> {
    fn truncate_ascii_whitespace_end(buf: &mut Vec<u8>) {
        while buf.last().is_some_and(|byte| !byte.is_ascii_alphanumeric()) {
            buf.pop();
        }
    }

    let mut output = std::process::Command::new("which").arg("gcloud").output()?;

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
