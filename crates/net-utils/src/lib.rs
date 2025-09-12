//! A collection of networking utility types, primarily geared towards working with [`tower`] +
//! [`tonic`].

pub mod backoff;
#[cfg(feature = "tonic")]
pub mod bidi2;
#[cfg(feature = "tonic")]
pub mod bidirec;
#[cfg(feature = "tonic")]
pub mod infallible;
pub mod once;
#[cfg(feature = "tonic")]
pub mod open_close;
#[cfg(feature = "tower")]
pub mod retry;

pub mod cloud_task_payload;

#[cfg(all(feature = "tonic", feature = "tower"))]
pub mod retry_bidi;

#[cfg(any(feature = "tonic", feature = "tower"))]
pub mod transient;

// pub mod json_stream;

/// Common builder methods for creating tonic channels
#[cfg(feature = "tonic")]
pub mod tonic_channel {
    use std::time::Duration;

    use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Error};

    pub async fn build_channel_with_defaults<E>(uri: &'static str) -> Result<Channel, E>
    where
        E: From<Error>,
    {
        ChannelBuilder::new(uri).connect().await.map_err(E::from)
    }

    #[derive(Debug)]
    pub struct ChannelBuilder {
        endpoint: Endpoint,
        domain: &'static str,
        tls_config: Option<ClientTlsConfig>,
    }

    fn get_domain_from_uri(uri: &str) -> &str {
        const HTTPS: &str = "https://";

        let Some(post_https) = uri.strip_prefix(HTTPS) else {
            panic!("expected a uri starting in 'https://' (got '{uri}')");
        };

        // cut off any any trailing path
        match post_https.split_once('/') {
            Some((domain, _path)) => domain,
            None => post_https,
        }
    }

    macro_rules! defer_to_endpoint {
        ($(
            $fn_name:ident($arg_name:ident : $($arg_ty:tt)*) $(-> Result<Self, $error_ty:ty>)?
        ),* $(,)?) => {
            $(
                defer_to_endpoint! {
                    @INNER
                    $fn_name($arg_name: $($arg_ty)*) $(-> Result<Self, $error_ty>)?
                }
            )*
        };
        (
            @INNER
            $fn_name:ident($arg_name:ident : $($arg_ty:tt)*)
        ) => {
            #[inline]
            #[doc = concat!("defers to [`tonic::transport::Endpoint::", stringify!($fn_name), "`]")]
            pub fn $fn_name(self, $arg_name: $($arg_ty)*) -> Self {
                self.with_endpoint(|endpoint| endpoint.$fn_name($arg_name))
            }
        };
        (
            @INNER
            $fn_name:ident($arg_name:ident : $($arg_ty:tt)*) -> Result<Self, $error_ty:ty>
        ) => {
            #[inline]
            #[doc = concat!("defers to [`tonic::transport::Endpoint::", stringify!($fn_name), "`]")]
            pub fn $fn_name(self, $arg_name: $($arg_ty)*) -> Result<Self, $error_ty> {
                self.try_with_endpoint(|endpoint| endpoint.$fn_name($arg_name))
            }
        };
    }

    impl ChannelBuilder {
        pub fn new(uri: &'static str) -> Self {
            Self {
                endpoint: Endpoint::from_static(uri),
                domain: get_domain_from_uri(uri),
                tls_config: None,
            }
        }

        defer_to_endpoint! {
            buffer_size(sz: impl Into<Option<usize>>),
            initial_connection_window_size(sz: impl Into<Option<u32>>),
            user_agent(user_agent: impl TryInto<http::HeaderValue>) -> Result<Self, Error>,
            timeout(dur: Duration),
            connect_timeout(dur: Duration),
            origin(origin: http::Uri),
            tcp_keepalive(tcp_keepalive: Option<Duration>),
            tcp_nodelay(enabled: bool),
            concurrency_limit(limit: usize),
            http2_adaptive_window(enabled: bool),
            http2_max_header_list_size(size: u32),
            http2_keep_alive_interval(interval: Duration),
            keep_alive_timeout(duration: Duration),
            keep_alive_while_idle(enabled: bool),
        }

        pub fn override_tls_config(mut self, tls_config: ClientTlsConfig) -> Self {
            self.tls_config = Some(tls_config);
            self
        }

        pub async fn connect(self) -> Result<Channel, Error> {
            self.into_endpoint()?.connect().await
        }

        pub fn into_endpoint(self) -> Result<Endpoint, Error> {
            let Self {
                endpoint,
                domain,
                tls_config,
            } = self;

            let tls_config = tls_config.unwrap_or_else(|| {
                ClientTlsConfig::new()
                    .domain_name(domain)
                    .with_enabled_roots()
            });

            endpoint.tls_config(tls_config)
        }

        fn with_endpoint(self, map_fn: impl FnOnce(Endpoint) -> Endpoint) -> Self {
            let Self {
                endpoint,
                domain,
                tls_config,
            } = self;

            let endpoint = map_fn(endpoint);

            Self {
                endpoint,
                domain,
                tls_config,
            }
        }

        fn try_with_endpoint<E>(
            self,
            map_fn: impl FnOnce(Endpoint) -> Result<Endpoint, E>,
        ) -> Result<Self, E> {
            let Self {
                endpoint,
                domain,
                tls_config,
            } = self;

            let endpoint = map_fn(endpoint)?;

            Ok(Self {
                endpoint,
                domain,
                tls_config,
            })
        }
    }
}
