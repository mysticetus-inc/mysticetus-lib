#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Transport(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    Auth(#[from] gcp_auth_provider::Error),
    #[error("{0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error("{0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("{0}")]
    Status(#[from] tonic::Status),
    #[error("{0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Internal Error: {0}")]
    Internal(&'static str),
}

impl From<gcp_auth_provider::channel::ChannelError> for Error {
    fn from(value: gcp_auth_provider::channel::ChannelError) -> Self {
        match value {
            gcp_auth_provider::channel::ChannelError::Auth(auth) => Self::Auth(auth),
            gcp_auth_provider::channel::ChannelError::Transport(e) => Self::from(e),
        }
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(value: tonic::transport::Error) -> Self {
        Self::Transport(Box::new(value))
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(task_err: tokio::task::JoinError) -> Self {
        error!(message = "tokio::task::JoinError encountered", error = %task_err);
        Self::Internal("task error")
    }
}
