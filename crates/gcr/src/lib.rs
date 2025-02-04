#![deny(clippy::suspicious, clippy::complexity, clippy::perf, clippy::style)]
#![feature(
    maybe_uninit_uninit_array,
    const_trait_impl,
    unboxed_closures,
    fn_traits
)]
//! Utilities for services running within the context of Cloud Run

#[macro_use]
extern crate tracing;

use std::net::SocketAddr;

pub mod active;
pub mod backoff;
pub mod body;
pub mod header;
pub mod init;
mod memory;
pub mod retry;
mod shutdown;
pub mod timeout;

pub use active::Active;
pub use init::{InitError, init_listener_and_state};
pub use memory::MemoryUsage;
pub use shutdown::Shutdown;

/// Lazily loads environment variable(s). This is a macro rather than a function to
/// statically format (via [`concat!`])
#[macro_export]
macro_rules! lazy_env {
    ($visib:vis $env_var:ident $(,)?) => {
        $vis static $env_var: $crate::std::sync::LazyLock<Box<str>> = $crate::std::sync::LazyLock::new(|| {
            $crate::std::env::var(stringify!($env_var))
                .expect(concat!("'$", stringify!($env_var), "' is not set"))
                .into_boxed_str()
        });
    };
    ($($visib:vis $env_var:ident),* $(,)?) => {
        $(
            $crate::lazy_env!($visib $env_var);
        )*
    };
}

/// Loads a token from the environment at compile time, and pre-formats a static [`HeaderValue`].
#[macro_export]
macro_rules! env_bearer_auth {
    ($env:literal) => {{ ::axum::http::header::HeaderValue::from_static(concat!("Bearer ", env!($env))) }};
}

/// Shorthand to grab + parse the $PORT environment variable.
/// Panics if $PORT is missing or is an invalid integer.
#[inline]
pub fn port() -> u16 {
    port_and_buf().0
}

/// Identical to [`port`], but also returns the allocated environment variable for re-use.
#[inline]
pub fn port_and_buf() -> (u16, String) {
    let buf = std::env::var("PORT").expect("no $PORT env var set");

    let port = buf.trim().parse::<u16>().expect("$PORT is an invalid u16");

    (port, buf)
}

/// Calls [`port`] and assembles a full [`SocketAddr`] at '0.0.0.0:$PORT'.
#[inline]
pub fn addr() -> SocketAddr {
    SocketAddr::V4(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(0, 0, 0, 0),
        port(),
    ))
}

/// Similar to [`port`] and [`port_and_buf`], but constructs a [`SocketAddr`].
#[inline]
pub fn addr_and_buf() -> (SocketAddr, String) {
    let (port, buf) = port_and_buf();
    let addr = SocketAddr::V4(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(0, 0, 0, 0),
        port,
    ));

    (addr, buf)
}

#[macro_export]
macro_rules! log_on_error {
    ($result:expr, $message:literal) => {{
        match $result {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(message = $message, error = ?error);
                return Err(error.into());
            }
        }
    }};
}
