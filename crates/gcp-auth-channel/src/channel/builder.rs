//! Builder struct for [`AuthChannel`]

use tonic::transport::Channel;

use super::{AuthChannel, Error};
use crate::Scope;
use crate::auth::Auth;

pub struct AuthChannelBuilder<C = (), S = (), A = ()> {
    channel: C,
    scope: S,
    auth: A,
}

impl Default for AuthChannelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthChannelBuilder {
    pub fn new() -> Self {
        Self {
            channel: (),
            scope: (),
            auth: (),
        }
    }
}

impl<C, S, A> AuthChannelBuilder<C, S, A> {
    pub fn with_channel(self, channel: Channel) -> AuthChannelBuilder<Channel, S, A> {
        AuthChannelBuilder {
            channel,
            scope: self.scope,
            auth: self.auth,
        }
    }

    pub fn with_scope(self, scope: Scope) -> AuthChannelBuilder<C, Scope, A> {
        AuthChannelBuilder {
            channel: self.channel,
            scope,
            auth: self.auth,
        }
    }

    pub fn with_auth<M>(self, auth: M) -> AuthChannelBuilder<C, S, Auth>
    where
        M: Into<Auth>,
    {
        AuthChannelBuilder {
            channel: self.channel,
            scope: self.scope,
            auth: auth.into(),
        }
    }
}

impl<C> AuthChannelBuilder<C, Scope, ()> {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_service_account_file<P>(
        self,
        project_id: &'static str,
        path: P,
    ) -> Result<AuthChannelBuilder<C, Scope, Auth>, Error>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(AuthChannelBuilder {
            channel: self.channel,
            auth: Auth::new_from_service_account_file(project_id, path.as_ref(), self.scope)?,
            scope: self.scope,
        })
    }
}

impl<S> AuthChannelBuilder<Channel, S, Auth> {
    pub fn build(self) -> AuthChannel {
        AuthChannel {
            auth: self.auth,
            svc: self.channel,
        }
    }
}

pub trait ClientBuilder: Sized {
    type Error: From<crate::Error>;

    type ChannelOptions: Send + 'static;

    fn new(project_id: &'static str, scope: Scope) -> Result<Self, Error>;

    fn build_channel(
        channel_opts: Self::ChannelOptions,
    ) -> impl Future<Output = Result<Channel, tonic::transport::Error>> + Send + 'static;
}
