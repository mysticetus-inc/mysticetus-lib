use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::TryMaybeDone;

#[cfg(feature = "application-default")]
use super::application_default;
#[cfg(feature = "emulator")]
use super::emulator;
#[cfg(feature = "gcloud")]
use super::gcloud;
use super::{metadata, service_account};
use crate::providers::{BaseTokenProvider, LoadProviderResult, ScopedTokenProvider, TokenProvider};
use crate::{Error, GetTokenFuture, ProjectId, Result, Scopes, client};

#[derive(Debug)]
pub enum Provider {
    #[cfg(feature = "application-default")]
    ApplicationDefault(application_default::ApplicationDefault),
    #[cfg(feature = "emulator")]
    Emulator(emulator::EmulatorProvider),
    #[cfg(feature = "gcloud")]
    GCloud(gcloud::GCloudProvider),
    MetadataServer(metadata::MetadataServer),
    ServiceAccount(service_account::ServiceAccount),
}

#[derive(Debug)]
pub enum UnscopedProvider {
    #[cfg(feature = "application-default")]
    ApplicationDefault(ProjectId, application_default::ApplicationDefault),
    ServiceAccount(ProjectId, service_account::ServiceAccount),
}

impl UnscopedProvider {
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::ServiceAccount(_, svc) => svc.name(),
            #[cfg(feature = "application-default")]
            Self::ApplicationDefault(_, app_def) => app_def.name(),
        }
    }

    pub fn project_id(&self) -> &ProjectId {
        match self {
            Self::ServiceAccount(proj_id, _) => proj_id,
            #[cfg(feature = "application-default")]
            Self::ApplicationDefault(proj_id, _) => proj_id,
        }
    }

    pub fn with_scope(self, scopes: impl Into<Scopes>) -> crate::Auth {
        match self {
            Self::ServiceAccount(project_id, svc) => {
                crate::Auth::new_from_provider(LoadProviderResult {
                    provider: svc.with_scopes(scopes.into()),
                    project_id,
                    token_future: TryMaybeDone::Gone,
                })
            }
            #[cfg(feature = "application-default")]
            Self::ApplicationDefault(project_id, app_def) => {
                crate::Auth::new_from_provider(LoadProviderResult {
                    provider: app_def.with_scopes(scopes.into()),
                    project_id,
                    token_future: TryMaybeDone::Gone,
                })
            }
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct InitContext {
    pub(crate) http: Option<client::HttpClient>,
    pub(crate) https: Option<client::HttpsClient>,
}

impl Provider {
    #[cfg(feature = "emulator")]
    pub const fn new_emulator() -> Self {
        Self::Emulator(emulator::EmulatorProvider)
    }

    // TODO: see if we can rework InitContext to avoid the static future lifetime
    pub fn detect() -> DetectFuture<'static> {
        let mut ctx = InitContext::default();

        // try and find a service account first, if the env var isn't set fallback
        // to looking for the metadata server.
        let state = match service_account::ServiceAccount::try_load(&mut ctx) {
            Some(fut) => DetectState::ServiceAccount {
                fut: fut.take_into_static(),
            },
            None => DetectState::Metadata {
                fut: metadata::MetadataServer::try_load(&mut ctx).into_static(),
            },
        };

        DetectFuture {
            ctx,
            log_errors: true,
            state,
        }
    }

    pub fn token_provider_name(&self) -> &'static str {
        match self {
            #[cfg(feature = "application-default")]
            Self::ApplicationDefault(app) => app.name(),
            #[cfg(feature = "emulator")]
            Self::Emulator(emulator) => emulator.name(),
            #[cfg(feature = "gcloud")]
            Self::GCloud(gcloud) => gcloud.name(),
            Self::MetadataServer(ms) => ms.name(),
            Self::ServiceAccount(svc) => svc.name(),
        }
    }

    pub fn as_token_provider(&self) -> Option<&dyn super::TokenProvider> {
        match self {
            #[cfg(feature = "gcloud")]
            Self::GCloud(gcloud) => Some(gcloud),
            #[cfg(feature = "emulator")]
            Self::Emulator(emulator) => Some(emulator),
            Self::MetadataServer(meta) => Some(meta),
            _ => None,
        }
    }

    pub fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_> {
        match self {
            #[cfg(feature = "application-default")]
            Self::ApplicationDefault(app) => app.get_scoped_token(scopes),
            #[cfg(feature = "gcloud")]
            Self::GCloud(gcloud) => gcloud.get_token(),
            #[cfg(feature = "emulator")]
            Self::Emulator(emulator) => emulator.get_token(),
            Self::MetadataServer(meta) => meta.get_token(),
            Self::ServiceAccount(acct) => acct.get_scoped_token(scopes),
        }
    }
}

impl super::BaseTokenProvider for Provider {
    fn name(&self) -> &'static str {
        self.token_provider_name()
    }
}

impl super::ScopedTokenProvider for Provider {
    #[inline]
    fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_> {
        self.get_scoped_token(scopes)
    }
}

pin_project_lite::pin_project! {
    pub struct DetectFuture<'a> {
        ctx: InitContext,
        log_errors: bool,
        #[pin]
        state: DetectState<'a>,
    }
}

fn maybe_log_error(log_errors: bool, provider_name: &'static str, error: Option<Error>) {
    if !log_errors {
        return;
    }

    let Some(error) = error else {
        return;
    };

    if !tracing::enabled!(tracing::Level::WARN) {
        return;
    }

    tracing::warn!(
        message = format_args!("failed to load {provider_name}"),
        error.display = %error,
        error = &error as &dyn std::error::Error,
    );
}

impl<'a> Future for DetectFuture<'a> {
    type Output = Result<LoadProviderResult<'static, Provider>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match std::task::ready!(this.state.poll_next_state(this.ctx, cx)) {
                NextState::Found(res) => return Poll::Ready(Ok(res)),
                NextState::Next {
                    next_state,
                    failed_prov_name,
                    error,
                } => {
                    maybe_log_error(*this.log_errors, failed_prov_name, error);
                    this.state.set(next_state);
                }
                NextState::NoneLeft {
                    failed_prov_name,
                    error,
                } => {
                    maybe_log_error(*this.log_errors, failed_prov_name, error);
                    return Poll::Ready(Err(Error::NoProviderFound));
                }
            }
        }
    }
}

// the macro in pin-project-lite can't handle variants with
// a #[cfg(...)] attribute on it properly, so we need to use the
// full pin_project proc macro. This ends up being a lot cleaner than
// solutions to force using pin-project-lite (either multiple definitions,
// or funky type aliases)
#[pin_project::pin_project(project = DetectStateProjection)]
enum DetectState<'a> {
    ServiceAccount {
        #[pin]
        fut: service_account::TryLoadFuture<'a>,
    },
    Metadata {
        #[pin]
        fut: metadata::TryLoadFuture<'a>,
    },
    #[cfg(feature = "application-default")]
    ApplicationDefault {
        #[pin]
        fut: application_default::TryLoadFuture<'a>,
    },
    #[cfg(feature = "gcloud")]
    GCloud {
        #[pin]
        fut: gcloud::GCloudCandidates,
    },
}

enum NextState<'a> {
    Found(LoadProviderResult<'static, Provider>),
    Next {
        next_state: DetectState<'a>,
        failed_prov_name: &'static str,
        error: Option<Error>,
    },
    NoneLeft {
        failed_prov_name: &'static str,
        error: Option<Error>,
    },
}

impl<'a> DetectState<'a> {
    fn poll_next_state(
        self: &mut Pin<&mut Self>,
        init_ctx: &mut InitContext,
        cx: &mut Context<'_>,
    ) -> Poll<NextState<'a>> {
        use DetectStateProjection::*;

        match self.as_mut().project() {
            ServiceAccount { mut fut } => match std::task::ready!(fut.as_mut().poll(cx)) {
                Ok(prov_res) => Poll::Ready(NextState::Found(
                    prov_res.map_provider(Provider::ServiceAccount),
                )),
                Err(error) => {
                    if let Some(client) = fut.take_client() {
                        init_ctx.https = Some(client);
                    }

                    Poll::Ready(NextState::Next {
                        next_state: DetectState::Metadata {
                            fut: metadata::MetadataServer::try_load(init_ctx),
                        },
                        failed_prov_name: "ServiceAccount",
                        error: Some(error),
                    })
                }
            },
            Metadata { mut fut } => {
                let error = match std::task::ready!(fut.as_mut().poll(cx)) {
                    Ok(Some(prov_res)) => {
                        return Poll::Ready(NextState::Found(
                            prov_res.map_provider(Provider::MetadataServer),
                        ));
                    }
                    Ok(None) => None,
                    Err(error) => Some(error),
                };

                if let Some(client) = fut.pin_take_http() {
                    init_ctx.http = Some(client);
                }

                #[cfg(feature = "application-default")]
                if let Some(next_fut) = application_default::ApplicationDefault::try_load(init_ctx)
                {
                    return Poll::Ready(NextState::Next {
                        next_state: DetectState::ApplicationDefault {
                            fut: next_fut.into_static(),
                        },
                        failed_prov_name: "MetadataServer",
                        error,
                    });
                }

                #[cfg(feature = "gcloud")]
                if let Some(fut) = gcloud::GCloudCandidates::find_from_path(true) {
                    return Poll::Ready(NextState::Next {
                        next_state: DetectState::GCloud { fut },
                        failed_prov_name: "MetadataServer",
                        error,
                    });
                }

                // if none of the above features are enabled, or neither returns
                // a future, we're out of provider options
                Poll::Ready(NextState::NoneLeft {
                    failed_prov_name: "MetadataServer",
                    error,
                })
            }
            #[cfg(feature = "application-default")]
            ApplicationDefault { mut fut } => {
                let error = match std::task::ready!(fut.as_mut().poll(cx)) {
                    Ok(Some((prov, project_id))) => {
                        return Poll::Ready(NextState::Found(LoadProviderResult {
                            provider: Provider::ApplicationDefault(prov),
                            project_id,
                            token_future: TryMaybeDone::Gone,
                        }));
                    }
                    Ok(None) => None,
                    Err(error) => Some(error),
                };

                if let Some(client) = fut.take_client() {
                    init_ctx.https = Some(client);
                }

                #[cfg(feature = "gcloud")]
                if let Some(fut) = gcloud::GCloudCandidates::find_from_path(true) {
                    return Poll::Ready(NextState::Next {
                        next_state: DetectState::GCloud { fut },
                        failed_prov_name: "ApplicationDefault",
                        error,
                    });
                }

                // if none of the above features are enabled, or neither returns
                // a future, we're out of provider options
                Poll::Ready(NextState::NoneLeft {
                    failed_prov_name: "ApplicationDefault",
                    error,
                })
            }
            #[cfg(feature = "gcloud")]
            GCloud { mut fut } => match std::task::ready!(fut.as_mut().poll(cx)) {
                Ok(Some((prov, project_id))) => Poll::Ready(NextState::Found(LoadProviderResult {
                    provider: Provider::GCloud(prov),
                    project_id,
                    token_future: TryMaybeDone::Gone,
                })),
                Ok(None) => Poll::Ready(NextState::NoneLeft {
                    failed_prov_name: "GCloud",
                    error: None,
                }),
                Err(error) => Poll::Ready(NextState::NoneLeft {
                    failed_prov_name: "GCloud",
                    error: Some(error.into()),
                }),
            },
        }
    }
}
