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

use std::{future::Future, net::SocketAddr};

pub mod active;
pub mod backoff;
pub mod header;
mod memory;
pub mod retry;
mod shutdown;
pub mod timeout;

pub use active::Active;
pub use memory::MemoryUsage;
pub use shutdown::Shutdown;
use tokio::net::TcpListener;

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
    ($env:literal) => {{
        ::axum::http::header::HeaderValue::from_static(concat!("Bearer ", env!($env)))
    }};
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

pub async fn initialize_listener_and_state<State, StateErr, OutputErr>(
    initialize_state_future: impl Future<Output = Result<State, StateErr>>,
) -> Result<(SocketAddr, TcpListener, State), OutputErr>
where
    OutputErr: From<std::io::Error> + From<StateErr>,
{
    use futures::future::{try_select, Either};

    let addr = addr();

    let listener_future = std::pin::pin!(TcpListener::bind(addr));
    let state_future = std::pin::pin!(initialize_state_future);

    match try_select(listener_future, state_future).await {
        Ok(Either::Left((listener, state_fut))) => {
            let state = state_fut.await.map_err(OutputErr::from)?;
            Ok((addr, listener, state))
        }
        Ok(Either::Right((state, listener_fut))) => {
            let listener = listener_fut.await.map_err(OutputErr::from)?;
            Ok((addr, listener, state))
        }
        Err(Either::Left((error, _))) => Err(OutputErr::from(error)),
        Err(Either::Right((error, _))) => Err(OutputErr::from(error)),
    }
}
