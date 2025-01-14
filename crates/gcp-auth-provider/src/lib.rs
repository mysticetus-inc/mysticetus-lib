mod application_default;
mod error;
mod gcloud;
mod metadata;
mod state;
mod token;
use std::future::Future;
use std::sync::Arc;

pub use error::Error;
pub use token::Token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenProvider {
    #[cfg(feature = "gcloud")]
    GCloud(gcloud::GCloudProvider),
}

trait RawTokenProvider: Sized {
    const NAME: &'static str;

    fn try_load() -> impl Future<Output = Result<Option<Self>>> + Send + 'static;

    fn project_id(&self) -> impl Future<Output = Result<Arc<str>>> + Send + 'static;

    fn get_token(&self) -> impl Future<Output = Result<Token>> + Send + 'static;
}
