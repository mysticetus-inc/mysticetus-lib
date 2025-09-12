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

    pub(crate) fn get_token_inner(&self) -> GCloudFuture<crate::Token> {
        // make the Command builder outside of the task, that way we can avoid
        // an unneeded Arc<Path> clone
        let mut cmd = std::process::Command::new(self.cmd_path.as_os_str());

        GCloudFuture(tokio::task::spawn_blocking(move || {
            let output = cmd
                .arg("auth")
                .arg("print-access-token")
                .arg("--quiet") // not that this actually does anything
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()?;

            let access_token = get_relevant_gcloud_output(output)?;

            crate::Token::new_with_default_expiry_time(&access_token)
                .map_err(crate::Error::invalid_data)
        }))
    }
}

// since this shells out to a new process, blocking impl's are actually preferred
// so run them in a task.
#[derive(Debug)]
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

fn query_project_id(cmd_path: &Path) -> std::io::Result<ProjectId> {
    let output = std::process::Command::new(&cmd_path)
        .arg("config")
        .arg("get-value")
        .arg("project")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()?;

    output.status.exit_ok().map_err(std::io::Error::other)?;

    let project_id = ProjectId::from(get_relevant_gcloud_output(output)?);
    Ok(project_id)
}

impl GCloudProvider {
    pub fn try_load(log_candidate_errors: bool) -> Option<GCloudCandidates> {
        GCloudCandidates::find_from_path(log_candidate_errors)
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
        crate::GetTokenFuture::new_gcloud(self)
    }
}

pub struct GCloudCandidates {
    log_errors: bool,
    join_set: tokio::task::JoinSet<Option<(PathBuf, std::io::Result<ProjectId>)>>,
}

impl Future for GCloudCandidates {
    type Output = std::io::Result<Option<(GCloudProvider, ProjectId)>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().poll(cx).map_ok(|opt| {
            opt.map(|(path, proj_id)| {
                let provider = GCloudProvider {
                    cmd_path: Arc::from(path),
                };

                (provider, proj_id)
            })
        })
    }
}

impl GCloudCandidates {
    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<Option<(PathBuf, ProjectId)>>> {
        loop {
            match std::task::ready!(self.join_set.poll_join_next(cx)) {
                Some(Ok(Some((path, Ok(project_id))))) => {
                    self.join_set.abort_all();
                    return Poll::Ready(Ok(Some((path, project_id))));
                }
                Some(Ok(Some((path, Err(error))))) => {
                    if self.log_errors {
                        if tracing::enabled!(tracing::Level::INFO) {
                            tracing::info!(
                                message = "[gcp-auth-provider::gcloud] error checking for gcloud executable",
                                path = %path.display(),
                                ?error,
                            );
                        } else {
                            println!(
                                "[gcp-auth-provider::gcloud] error checking possible gcloud exe @ \
                                 {path:?}: {error}"
                            );
                        }
                    }
                }
                Some(Ok(None)) => continue,
                Some(Err(join_err)) => return Poll::Ready(Err(std::io::Error::other(join_err))),
                None => return Poll::Ready(Ok(None)),
            }
        }
    }

    pub fn find_from_path(log_errors: bool) -> Option<Self> {
        let Some(path) = std::env::var_os("PATH") else {
            return None;
        };

        if path.is_empty() {
            return None;
        }

        let mut join_set = tokio::task::JoinSet::new();

        for mut path in std::env::split_paths(path.as_os_str()) {
            join_set.spawn_blocking(move || {
                path.push("gcloud");

                let meta = match std::fs::metadata(&path) {
                    Ok(meta) => meta,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
                    Err(error) => return Some((path, Err(error))),
                };

                if !meta.is_file() {
                    return None;
                }

                #[cfg(target_family = "unix")]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = meta.permissions();
                    // see if the file is executable
                    if perms.mode() & 0o111 == 0 {
                        return None;
                    }
                }

                let result = query_project_id(&path);
                Some((path, result))
            });
        }

        if join_set.is_empty() {
            return None;
        }

        Some(Self {
            log_errors,
            join_set,
        })
    }
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
        return Err(std::io::Error::other(OsStringError(OsString::from_vec(
            output.stderr,
        ))));
    }

    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_gcloud_from_path() -> crate::Result<()> {
        let find = GCloudCandidates::find_from_path(true).unwrap();
        let (prov, project_id) = find.await?.unwrap();
        println!("{prov:?} - {project_id}");
        Ok(())
    }
}
