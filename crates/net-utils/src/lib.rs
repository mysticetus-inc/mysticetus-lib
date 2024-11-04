//! A collection of networking utility types, primarily geared towards working with [`tower`] +
//! [`tonic`].

pub mod backoff;
#[cfg(feature = "tonic")]
pub mod bidirec;
pub mod once;
#[cfg(feature = "tonic")]
pub mod open_close;
#[cfg(feature = "tower")]
pub mod retry;
