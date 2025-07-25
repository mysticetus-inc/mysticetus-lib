use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Error as TransportError};

use crate::providers::UnscopedProvider;
use crate::service::AuthSvc;
use crate::{Auth, Scopes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthChannelBuilder<A, C> {
    auth: A,
    channel: C,
    scopes: Scopes,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Auth(#[from] crate::Error),
}

impl From<std::convert::Infallible> for ChannelError {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

impl Default for AuthChannelBuilder<(), ()> {
    #[inline]
    fn default() -> Self {
        Self {
            auth: (),
            channel: (),
            scopes: Scopes::empty(),
        }
    }
}

impl<Chan> AuthChannelBuilder<(), Chan> {
    pub fn auth<A>(self, auth: A) -> AuthChannelBuilder<A, Chan> {
        AuthChannelBuilder {
            auth,
            channel: self.channel,
            scopes: self.scopes,
        }
    }
}

impl AuthChannelBuilder<crate::AuthDetectFuture, ()> {
    pub fn new_detect() -> Self {
        Self {
            auth: crate::Auth::new_detect(),
            scopes: Scopes::empty(),
            channel: (),
        }
    }
}

impl AuthChannelBuilder<crate::AuthMetadataFuture, ()> {
    pub fn new_metadata() -> Self {
        Self {
            auth: crate::Auth::new_metadata(),
            scopes: Scopes::empty(),
            channel: (),
        }
    }
}

impl<A, C> AuthChannelBuilder<A, C> {
    pub fn scopes_mut(&mut self) -> &mut Scopes {
        &mut self.scopes
    }

    pub fn add_scopes(mut self, scopes: impl Into<Scopes>) -> Self {
        self.scopes.extend(scopes.into());
        self
    }

    pub fn cloud_platform_admin(mut self) -> Self {
        self.scopes.insert(Scopes::CLOUD_PLATFORM_ADMIN);
        self
    }
}

impl<A> AuthChannelBuilder<A, ()> {
    pub fn channel<Chan>(self, channel: Chan) -> AuthChannelBuilder<A, Chan> {
        AuthChannelBuilder {
            auth: self.auth,
            channel,
            scopes: self.scopes,
        }
    }

    pub fn channel_builder(self, uri: &'static str) -> AuthChannelBuilder<A, ChannelOptions> {
        AuthChannelBuilder {
            auth: self.auth,
            scopes: self.scopes,
            channel: ChannelOptions::new(uri),
        }
    }

    pub fn channel_with_defaults(
        self,
        uri: &'static str,
        domain: impl Into<Option<&'static str>>,
    ) -> AuthChannelBuilder<A, ChannelOptions> {
        let mut channel = ChannelOptions::new(uri);

        if let Some(domain) = domain.into() {
            channel.domain(domain);
        }

        channel.default_tls();

        AuthChannelBuilder {
            auth: self.auth,
            scopes: self.scopes,
            channel,
        }
    }
}

impl AuthChannelBuilder<Auth, Channel> {
    pub fn build(self) -> AuthSvc<Channel> {
        AuthSvc {
            auth: self.auth,
            svc: self.channel,
        }
    }
}

impl AuthChannelBuilder<crate::AuthDetectFuture, ChannelOptions> {
    pub async fn build(self) -> Result<AuthSvc<Channel>, ChannelError> {
        let (result, svc) = tokio::try_join!(
            async move { self.auth.await.map_err(Into::<ChannelError>::into) },
            self.channel.connect_map_err()
        )?;

        match result {
            Ok(auth) => Ok(AuthSvc { auth, svc }),
            Err(unscoped) => Ok(AuthSvc {
                auth: unscoped_to_auth(unscoped, self.scopes),
                svc,
            }),
        }
    }
}

fn unscoped_to_auth(unscoped: UnscopedProvider, scopes: Scopes) -> Auth {
    // if there are specified scopes, we're good, bail early.
    if !scopes.is_empty() {
        return unscoped.with_scope(scopes);
    }

    // in prod, if this is a service account, just fallback to CloudPlatformAdmin
    // to avoid errors, but do include a warning (that alerts, since this should be
    // considered a fairly urgent config error).
    #[cfg(not(debug_assertions))]
    if matches!(unscoped, UnscopedProvider::ServiceAccount(_, _)) {
        // ensure that if there's no logging set up, we fallback to emitting to stderr.
        if tracing::enabled!(tracing::Level::WARN) {
            tracing::warn!(
                message = "FIXME: using admin fallback scope for service account auth",
                project_id = %unscoped.project_id(),
                alert = true,
            );
        } else {
            eprintln!(
                "FIXME: using admin fallback scope for service account auth: {}",
                unscoped.project_id()
            );
        }

        return unscoped.with_scope(Scopes::CLOUD_PLATFORM_ADMIN);
    }

    // otherwise, this should panic, but throw out a warning first, using the same logging
    // fallback logic as above
    if tracing::enabled!(tracing::Level::WARN) {
        tracing::warn!(
            message = "FIXME: no scopes specified for provider",
            project_id = %unscoped.project_id(),
            provider_name = unscoped.provider_name(),
            alert = true,
        );
    } else {
        eprintln!(
            "FIXME: no scopes specified for {} provider (project {})",
            unscoped.provider_name(),
            unscoped.project_id(),
        );
    }

    panic!();
}

impl<F> AuthChannelBuilder<F, ChannelOptions>
where
    F: futures::TryFuture<Ok = Auth, Error: Into<ChannelError>>,
    F: Future<Output = std::result::Result<F::Ok, F::Error>>,
{
    pub async fn build(self) -> Result<AuthSvc<Channel>, ChannelError> {
        let (auth, svc) = tokio::try_join!(
            async move { self.auth.await.map_err(Into::<ChannelError>::into) },
            self.channel.connect_map_err()
        )?;
        Ok(AuthSvc { auth, svc })
    }
}

impl AuthChannelBuilder<Auth, ChannelOptions> {
    pub async fn build(self) -> Result<AuthSvc<Channel>, ChannelError> {
        let channel = self.channel.connect().await?;

        Ok(AuthSvc {
            auth: self.auth,
            svc: channel,
        })
    }
}

pub struct ChannelOptions {
    uri: &'static str,
    domain: Option<&'static str>,
    user_agent: Option<&'static str>,
    tls_config: Option<ClientTlsConfig>,
}

impl ChannelOptions {
    pub const fn new(uri: &'static str) -> Self {
        Self {
            uri,
            user_agent: None,
            domain: None,
            tls_config: None,
        }
    }

    pub const fn domain(&mut self, domain: &'static str) -> &mut Self {
        self.domain = Some(domain);
        self
    }
    pub const fn user_agent(&mut self, user_agent: &'static str) -> &mut Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn tls_config(&mut self, tls_config: ClientTlsConfig) -> &mut Self {
        self.tls_config = Some(tls_config);
        self
    }

    pub fn default_tls(&mut self) -> &mut Self {
        let mut config = ClientTlsConfig::new().with_enabled_roots();
        if let Some(domain) = self.get_or_extract_domain() {
            config = config.domain_name(domain);
        }

        self.tls_config(config)
    }

    fn get_or_extract_domain(&self) -> Option<&'static str> {
        if let Some(domain) = self.domain {
            return Some(domain);
        }

        let no_scheme = self.uri.strip_prefix("https://")?;

        let extracted_domain = no_scheme
            .split_once('/')
            .map(|(domain, _)| domain)
            .unwrap_or(no_scheme);

        Some(extracted_domain)
    }

    pub fn build_endpoint(mut self) -> Result<Endpoint, TransportError> {
        let mut endpoint = Channel::from_static(self.uri);

        if let Some(user_agent) = self.user_agent {
            endpoint = endpoint.user_agent(user_agent)?;
        }

        if let Some(tls) = self.tls_config.take() {
            endpoint = endpoint.tls_config(tls)?;
        }

        Ok(endpoint)
    }

    async fn connect_map_err(self) -> Result<Channel, ChannelError> {
        self.build_endpoint()?
            .connect()
            .await
            .map_err(ChannelError::Transport)
    }

    pub async fn connect(self) -> Result<Channel, TransportError> {
        self.build_endpoint()?.connect().await
    }

    pub fn connect_lazy(self) -> Result<Channel, TransportError> {
        Ok(self.build_endpoint()?.connect_lazy())
    }
}
