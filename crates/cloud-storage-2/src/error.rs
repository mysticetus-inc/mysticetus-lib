#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Transport(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    Auth(#[from] gcp_auth_channel::GcpAuthError),
    #[error("{0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("{0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error(transparent)]
    InvalidMetadataName(#[from] tonic::metadata::errors::InvalidMetadataValue),
    #[error("{0}")]
    Status(#[from] tonic::Status),
}

impl From<gcp_auth_channel::Error> for Error {
    fn from(auth: gcp_auth_channel::Error) -> Self {
        match auth {
            gcp_auth_channel::Error::Auth(auth) => Self::Auth(auth),
            gcp_auth_channel::Error::Transport(transport) => Self::Transport(transport),
            gcp_auth_channel::Error::InvalidHeaderName(name) => Self::InvalidHeaderName(name),
            gcp_auth_channel::Error::InvalidHeaderValue(val) => Self::InvalidHeaderValue(val),
        }
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        gcp_auth_channel::Error::from(err).into()
    }
}
