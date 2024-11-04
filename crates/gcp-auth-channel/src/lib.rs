#![feature(const_trait_impl, trait_alias, variant_count, once_cell_try)]

/// Re-export [`gcp_auth::Error`] so consumers of this crate can unpack [`Error`] into their
/// own types, while avoiding the need to add [`gcp_auth`] itself as a dependency.
pub use gcp_auth::Error as GcpAuthError;

pub type Result<T> = core::result::Result<T, Error>;

pub mod auth;
pub mod scope;

pub use auth::Auth;
pub use scope::Scope;

#[cfg(feature = "channel")]
pub mod channel;
#[cfg(feature = "channel")]
pub use channel::AuthChannel;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth::Error),
    #[error(transparent)]
    #[cfg(feature = "channel")]
    Transport(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    InvalidHeaderName(http::header::InvalidHeaderName),
    #[error(transparent)]
    InvalidHeaderValue(http::header::InvalidHeaderValue),
}

#[cfg(feature = "channel")]
impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Self::Transport(err.into())
    }
}
