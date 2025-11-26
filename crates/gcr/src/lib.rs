#![deny(clippy::suspicious, clippy::complexity, clippy::perf, clippy::style)]
#![feature(const_trait_impl, unboxed_closures, fn_traits)]
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

/// Returns the current stage, detected from either '#[cfg(test)]',
/// or the 'K_SERVICE' environment variable (via [is_dev]).
#[inline]
pub fn stage() -> gcp_logging::Stage {
    #[cfg(test)]
    {
        gcp_logging::Stage::Test
    }
    #[cfg(not(test))]
    {
        if is_dev() {
            gcp_logging::Stage::Dev
        } else {
            gcp_logging::Stage::Production
        }
    }
}

/// Checks if we're currently running in a prod environment, by calling <code>!is_dev()</code>
#[inline]
pub fn is_prod() -> bool {
    !is_dev()
}

/// Checks if we're currently running in a dev environment.
/// Keys off of the 'K_SERVICE' environment variable that should always exist
/// within a cloud run context. If this variable doesn't exist, or its value
/// doesn't end with "-dev", this returns true.
///
/// Result is cached, so calling this multiple times won't incur overhead
#[inline]
pub fn is_dev() -> bool {
    static IS_DEV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

    fn check_is_dev() -> bool {
        match std::env::var_os("K_SERVICE") {
            Some(svc) => svc.as_encoded_bytes().trim_ascii().ends_with(b"-dev"),
            // if this env var doesn't exist, we aren't
            // running in cloud run.
            None => true,
        }
    }

    *IS_DEV.get_or_init(check_is_dev)
}

mod serve {
    #![allow(dead_code)]
    use std::convert::Infallible;

    use axum::Router;
    use axum::extract::Request;
    use axum::response::{IntoResponse, Response};
    use tower_service::Service;

    #[derive(Debug, Clone)]
    struct StateHandler<S, F> {
        state: S,
        handler: F,
    }

    #[rustfmt::skip]
    macro_rules! all_the_tuples {
        ($name:ident) => {
            $name!([], T1);
            $name!([T1], T2);
            $name!([T1, T2], T3);
            $name!([T1, T2, T3], T4);
            $name!([T1, T2, T3, T4], T5);
            $name!([T1, T2, T3, T4, T5], T6);
            $name!([T1, T2, T3, T4, T5, T6], T7);
            $name!([T1, T2, T3, T4, T5, T6, T7], T8);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
            $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
        };
    }

    macro_rules! impl_handler {
        (
            [$($ty:ident),*], $last:ident
        ) => {
            #[allow(non_snake_case, unused_mut)]
            impl<F, Fut, S, Res, M, $($ty,)* $last> axum::handler::Handler<(M, S, $($ty,)* $last,), ()> for StateHandler<S, F>
            where
                F: FnOnce(S, $($ty,)* $last,) -> Fut + Clone + Send + Sync + 'static,
                Fut: Future<Output = Res> + Send,
                S: Send + Sync + Clone + 'static,
                Res: IntoResponse,
                $( $ty: axum::extract::FromRequestParts<()> + Send, )*
                $last: axum::extract::FromRequest<(), M> + Send,
            {
                type Future = JoinHandleResponse<Response>;

                fn call(self, req: Request, _: ()) -> Self::Future {
                    let Self { state, handler } = self;

                    let (mut parts, body) = req.into_parts();
                    let handle = tokio::spawn(async move {
                        $(
                            let $ty = match $ty::from_request_parts(&mut parts, &()).await {
                                Ok(value) => value,
                                Err(rejection) => return rejection.into_response(),
                            };
                        )*

                        let req = Request::from_parts(parts, body);

                        let $last = match $last::from_request(req, &()).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };

                        (handler)(state, $($ty,)* $last,).await.into_response()
                    });

                    fn convert_error(error: tokio::task::JoinError) -> Response {
                        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
                    }

                    JoinHandleResponse(handle, convert_error)
                }
            }
        };
    }

    struct JoinHandleResponse<R>(
        tokio::task::JoinHandle<R>,
        fn(tokio::task::JoinError) -> Response,
    );

    impl<R: IntoResponse + Send + 'static> Future for JoinHandleResponse<R> {
        type Output = Response;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            let Self(handle, join_error_to_response) = self.get_mut();
            match std::task::ready!(std::pin::Pin::new(handle).poll(cx)) {
                Ok(inner) => std::task::Poll::Ready(inner.into_response()),
                Err(error) => std::task::Poll::Ready((join_error_to_response)(error)),
            }
        }
    }

    all_the_tuples!(impl_handler);

    pub async fn serve_with_shutdown<L, S, F, E>(
        _listener: L,
        _router: Router<S>,
        _shutdown_task: F,
    ) -> Result<(), super::InitError<E>>
    where
        Router<S>:
            Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
        F: AsyncFnOnce() -> Result<(), E>,
    {
        todo!()
    }
}

#[macro_export]
macro_rules! log_on_error {
    ($result:expr, $message:literal $(, $($error_arg:tt)*)?) => {{
        match $result {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(message = $message, error = ?error $(, $($error_arg)*)?);
                return Err(error.into());
            }
        }
    }};
}
