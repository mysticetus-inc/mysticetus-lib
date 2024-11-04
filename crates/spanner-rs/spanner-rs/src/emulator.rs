use core::fmt;
use std::process::Stdio;

use anyhow::anyhow;
use gcp_auth_channel::{Auth, AuthChannel};
use timestamp::Duration;
use tokio::io::AsyncBufReadExt;

/// The default port used by the spanner emulator for gRPC requests
const DEFAULT_EMULATOR_GRPC_PORT: u16 = 9010;

/// The default port used by the spanner emulator for REST requests
const DEFAULT_EMULATOR_REST_PORT: u16 = 9020;

const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_seconds(30);

const EMULATOR_DOCKER_TAG: &str = "gcr.io/cloud-spanner-emulator/emulator";

pub struct Emulator {
    child_proc: tokio::process::Child,
    options: EmulatorOptions,
    channel: AuthChannel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmulatorOptions {
    pub grpc_port: u16,
    pub rest_port: u16,
    pub enable_fault_injection: bool,
    pub run_docker_pull: bool,
    pub emulator_startup_timeout: Duration,
}

impl EmulatorOptions {
    /// Convieniece call for [`Emulator::start`], passing in `self`
    pub async fn start(self) -> crate::Result<Emulator> {
        Emulator::start(self).await
    }
}

impl Default for EmulatorOptions {
    fn default() -> Self {
        Self {
            grpc_port: DEFAULT_EMULATOR_GRPC_PORT,
            rest_port: DEFAULT_EMULATOR_REST_PORT,
            emulator_startup_timeout: DEFAULT_STARTUP_TIMEOUT,
            enable_fault_injection: false,
            run_docker_pull: true,
        }
    }
}

fn io_to_misc_err(error: std::io::Error, prefix: impl fmt::Display) -> crate::Error {
    crate::Error::Misc(anyhow!("{prefix}: {error}"))
}

async fn docker_pull_emulator_image() -> crate::Result<()> {
    // pull the emulator image
    let pull_output = tokio::process::Command::new("docker")
        .arg("pull")
        .arg(EMULATOR_DOCKER_TAG)
        .output()
        .await
        .map_err(|err| io_to_misc_err(err, "running `docker pull` failed"))?;

    if !pull_output.status.success() {
        // we don't capture stdout/err (though it might be a good idea later on)
        // so just indicate that the error message should be there
        return Err(crate::Error::Misc(anyhow!(
            "running `docker pull` failed (stdout/stderr should have more info)",
        )));
    }

    Ok(())
}

fn spawn_docker_run(options: &EmulatorOptions) -> crate::Result<tokio::process::Child> {
    let EmulatorOptions {
        grpc_port,
        rest_port,
        ..
    } = options;

    tokio::process::Command::new("docker")
        .arg("run")
        .arg("-t")
        .arg("-i") // -t, -d and -i all needed otherwise docker ignores to sigkills (lol)
        .arg("-p")
        .arg(format!("{rest_port}:{DEFAULT_EMULATOR_REST_PORT}"))
        .arg("-p")
        .arg(format!("{grpc_port}:{DEFAULT_EMULATOR_GRPC_PORT}"))
        .arg(EMULATOR_DOCKER_TAG)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| io_to_misc_err(err, "failed to run `docker run`"))
}

async fn wait_for_emulator_startup(
    child: &mut tokio::process::Child,
    options: &EmulatorOptions,
) -> crate::Result<()> {
    let stderr = child.stderr.as_mut().expect("'docker run' has no stdout");

    let mut lines = tokio::io::BufReader::new(stderr).lines();

    let wait_for_emulator_running_future = async move {
        while let Some(line) = lines.next_line().await? {
            println!("{line}");
            if line.contains("gRPC server listening at") {
                return Ok(());
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "docker run stderr closed, unknown if emulator is actually running",
        ))
    };

    // wrap the wait future with a timeout
    let timed_out_fut = tokio::time::timeout(
        options.emulator_startup_timeout.into(),
        wait_for_emulator_running_future,
    );

    match timed_out_fut.await {
        Ok(result) => {
            result.map_err(|err| io_to_misc_err(err, "error waiting for emulator to start up"))
        }
        Err(_timeout_err) => Err(crate::Error::Misc(anyhow::anyhow!(
            "timed out while waiting for the spanner emulator to start"
        ))),
    }
}

async fn build_emulator_channel(port: u16) -> crate::Result<tonic::transport::Channel> {
    const LOCALHOST_W_PORT_COLON: &[u8] = b"127.0.0.1:";

    let mut buf = itoa::Buffer::new();
    let port_str = buf.format(port);

    let mut authority_bytes =
        bytes::BytesMut::with_capacity(LOCALHOST_W_PORT_COLON.len() + port_str.len());
    authority_bytes.extend_from_slice(LOCALHOST_W_PORT_COLON);
    authority_bytes.extend_from_slice(port_str.as_bytes());

    let authority_bytes = authority_bytes.freeze();

    let authority = http::uri::Authority::from_maybe_shared(authority_bytes).unwrap();

    let uri = tonic::transport::Uri::builder()
        .scheme("http")
        .authority(authority)
        .path_and_query("/")
        .build()?;

    tonic::transport::Channel::builder(uri)
        .user_agent(gcp_auth_channel::user_agent!())?
        .connect()
        .await
        .map_err(crate::Error::from)
}

impl Emulator {
    pub async fn start(options: EmulatorOptions) -> crate::Result<Self> {
        if options.run_docker_pull {
            docker_pull_emulator_image().await?;
        }

        let mut child_proc = spawn_docker_run(&options)?;

        #[allow(unreachable_code)] // see comment in the spawned future
        fn drain_stdio<Src, Sink>(src: &mut Option<Src>, mut sink: Sink)
        where
            Src: tokio::io::AsyncReadExt + Unpin + Send + 'static,
            Sink: tokio::io::AsyncWriteExt + Unpin + Send + 'static,
        {
            let mut src = src.take().expect("should exist");
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];

                loop {
                    match src.read(&mut buf).await? {
                        0 => continue,
                        read => sink.write_all(&buf[..read]).await?,
                    }
                }

                // we'll never hit this, but we need it for type annotations
                // (because async type inference isnt perfect)
                Ok(()) as std::io::Result<()>
            });
        }

        drain_stdio(&mut child_proc.stderr, tokio::io::stderr());
        drain_stdio(&mut child_proc.stdout, tokio::io::stdout());

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        // wait_for_emulator_startup(&mut child_proc, &options).await?;

        let channel = build_emulator_channel(options.grpc_port).await?;

        let channel = AuthChannel::builder()
            .with_auth(Auth::new_emulator(
                "",
                gcp_auth_channel::Scope::SpannerAdmin,
            ))
            .with_channel(channel)
            .build();

        Ok(Self {
            child_proc,
            options,
            channel,
        })
    }

    #[cfg(feature = "admin")]
    pub fn admin_client(&self) -> crate::admin::SpannerAdmin {
        crate::admin::SpannerAdmin::from_channel(self.channel.clone())
    }

    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child_proc.stdout.take()
    }

    #[cfg(feature = "admin")]
    pub async fn create_database(
        &self,
        database: crate::Database,
        instance_compute: crate::admin::InstanceCompute,
        ddl_statements: Vec<String>,
        timeout: Option<Duration>,
    ) -> crate::Result<crate::Client> {
        let admin_client = self.admin_client();

        admin_client
            .create_instance(&database.as_instance_builder(), instance_compute)
            .await?
            .wait(timeout)
            .await?;

        admin_client
            .create_database(&database, ddl_statements)
            .await?
            .wait(timeout)
            .await?;

        Ok(crate::Client::from_parts(
            database,
            admin_client.into_channel(),
        ))
    }

    pub fn options(&self) -> &EmulatorOptions {
        &self.options
    }

    fn try_kill(&mut self, timeout: Duration) -> std::io::Result<std::process::ExitStatus> {
        if let Some(exit_status) = self.child_proc.try_wait()? {
            return Ok(exit_status);
        }

        self.child_proc.start_kill()?;
        let start = std::time::Instant::now();
        let timeout: std::time::Duration = timeout.into();

        while start.elapsed() < timeout {
            if let Some(exit_status) = self.child_proc.try_wait()? {
                return Ok(exit_status);
            }

            std::thread::yield_now();
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "timed out waiting for process to be killed",
        ))
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        match self.try_kill(Duration::from_seconds(15)) {
            Ok(status) => {
                if !status.success() {
                    eprintln!(
                        "failed to kill spanner emulator docker process, exited with code {:?}",
                        status.code()
                    );
                }
            }
            Err(error) => {
                eprintln!("failed to kill spanner emulator docker process: {error}");
            }
        }
    }
}
