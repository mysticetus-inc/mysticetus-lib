#![feature(const_trait_impl, trait_alias, variant_count, once_cell_try)]

/// Re-export [`gcp_auth::Error`] so consumers of this crate can unpack [`Error`] into their
/// own types, while avoiding the need to add [`gcp_auth`] itself as a dependency.
pub use gcp_auth::Error as GcpAuthError;

pub type Result<T> = core::result::Result<T, Error>;

pub mod auth;
pub mod scope;

pub use auth::Auth;
pub use scope::Scope;
// pub mod future;

#[cfg(feature = "channel")]
pub mod channel;
#[cfg(feature = "channel")]
pub use channel::AuthChannel;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth::Error),
    #[cfg(feature = "channel")]
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[cfg(feature = "channel")]
    #[error(transparent)]
    Channel(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    InvalidHeaderName(http::header::InvalidHeaderName),
    #[error(transparent)]
    InvalidHeaderValue(http::header::InvalidHeaderValue),
}

impl From<gcp_auth_provider::Error> for Error {
    fn from(value: gcp_auth_provider::Error) -> Self {
        // TODO: replace this with the Auth variant once gcp_auth is phased out
        Self::channel(value)
    }
}

impl Error {
    #[cfg(feature = "channel")]
    pub(crate) fn channel(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::Channel(error.into())
    }
}
