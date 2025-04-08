use std::ffi::OsString;
use std::future::Future;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(super) struct GCloudProvider {
    cmd_path: Arc<Path>,
    project_id: Arc<str>,
}

impl GCloudProvider {
    pub fn new(cmd_path: impl Into<PathBuf>) -> Self {
        Self {
            cmd_path: Arc::from(PathBuf::into_boxed_path(cmd_path.into())),
        }
    }
}

// spawning async processes for simple tasks is significantly slower than the std/blocking
// variants, so run them in a blocking task to avoid the overhead
#[inline]
async fn spawn_blocking<T>(
    blocking_op: impl FnOnce() -> crate::Result<T> + Send + 'static,
) -> crate::Result<T>
where
    T: Send + 'static,
{
    use std::io::{Error, ErrorKind};

    match tokio::task::spawn_blocking(blocking_op).await {
        Ok(result) => result,
        Err(err) if err.is_cancelled() => {
            Err(crate::Error::Io(Error::from(ErrorKind::Interrupted)))
        }
        Err(err) => Err(crate::Error::Io(Error::new(ErrorKind::Other, err))),
    }
}

impl super::RawTokenProvider for GCloudProvider {
    const NAME: &'static str = "gcloud";

    fn try_load() -> impl Future<Output = crate::Result<Option<Self>>> + Send + 'static {
        spawn_blocking(|| {
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

            let project_id = get_relevant_gcloud_output(output)?;

            Ok(Self {
                project_id: Arc::from(project_id),
                cmd_path: Arc::from(cmd_path),
            })
        })
    }

    fn get_token(
        &self,
        _scope: crate::Scopes,
    ) -> impl Future<Output = crate::Result<crate::Token>> + Send + 'static {
        let mut cmd = std::process::Command::new(&*self.cmd_path);

        spawn_blocking(move || {
            let output = cmd
                .arg("auth")
                .arg("print-access-token")
                .arg("--quiet") // not that this actually does anything
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()?;

            let access_token = get_relevant_gcloud_output(output)?.into_boxed_str();

            Ok(crate::Token::new_with_default_expiry_time(access_token))
        })
    }

    fn project_id(&self) -> &Arc<str> {
        &self.project_id
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
