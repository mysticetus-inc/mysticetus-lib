use std::pin::Pin;
use std::sync::LazyLock;
use std::task::{Context, Poll};

use futures::future::TryMaybeDone;
use http::{HeaderName, HeaderValue, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::connect::dns::GaiResolver;

use super::InitContext;
use super::future::Resolver;
use crate::client::HttpClient;
use crate::client::future::{RequestCollect, RequestJson};
use crate::providers::LoadProviderResult;
use crate::token::Bearer;
use crate::util::CowMut;
use crate::{Error, GetTokenFuture, ProjectId, Result, Token};

// static HOST: &str = "http://metadata.google.internal";

static TOKEN_URI: LazyLock<Uri> = LazyLock::new(|| {
    Uri::from_static("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
});

static PROJECT_ID_URI: LazyLock<Uri> = LazyLock::new(|| {
    Uri::from_static("http://metadata.google.internal/computeMetadata/v1/project/project-id")
});

const METADATA_FLAVOR_NAME: HeaderName = HeaderName::from_static("metadata-flavor");
const METADATA_FLAVOR_VALUE: HeaderValue = HeaderValue::from_static("Google");

#[derive(Debug, Clone)]
pub struct MetadataServer {
    client: crate::client::HttpClient,
}

impl MetadataServer {
    /// Initializes a new MetadataServer connection. Returns [`crate::Error::NoProviderFound`]
    /// if the server isn't reachable.
    pub fn new() -> LoadFuture<'static> {
        let client = crate::client::Client::new_http();
        LoadFuture::new(CowMut::Owned(Some(client)))
    }

    pub fn try_new() -> TryLoadFuture<'static> {
        LoadFuture::new(CowMut::Owned(None)).try_load()
    }

    pub(super) fn try_load(ctx: &mut InitContext) -> TryLoadFuture<'static> {
        LoadFuture::new(CowMut::RefMut(&mut ctx.http))
            .into_static()
            .try_load()
    }
}

impl super::BaseTokenProvider for MetadataServer {
    #[inline]
    fn name(&self) -> &'static str {
        "metadata server"
    }
}

impl super::TokenProvider for MetadataServer {
    fn get_token(&self) -> crate::GetTokenFuture<'_> {
        let request = make_request(&TOKEN_URI);
        crate::GetTokenFuture::new_http(&self.client, request)
    }
}

fn make_request(uri: &Uri) -> http::Request<crate::client::BytesBody> {
    http::Request::builder()
        .method(http::Method::GET)
        .uri(uri)
        .header(METADATA_FLAVOR_NAME, METADATA_FLAVOR_VALUE)
        .body(crate::client::BytesBody::empty())
        .expect("header/uri values are pre-parsed constants (i.e shouldn't error)")
}

/*
async fn lookup_metadata_host() -> std::io::Result<&'static [std::net::SocketAddr]> {
    static SOCKET_ADDRS: tokio::sync::OnceCell<Vec<std::net::SocketAddr>> =
        tokio::sync::OnceCell::const_new();

    SOCKET_ADDRS
        .get_or_try_init(|| async {
            let addrs = tokio::net::lookup_host((HOST, 80)).await?;

            Ok(addrs.collect::<Vec<_>>())
        })
        .await
        .map(|vec| vec.as_slice())
}
*/

pin_project_lite::pin_project! {
    pub struct ProjectIdRequest<'a, R: Resolver = GaiResolver> {
        #[pin]
        inner: RequestCollect<'a, HttpConnector<R>>
    }
}

impl<'a, R: Resolver> ProjectIdRequest<'a, R> {
    pub(crate) fn new(client: &'a HttpClient<R>) -> Self {
        Self {
            inner: client.request(make_request(&PROJECT_ID_URI)),
        }
    }

    pub(crate) fn into_static(self) -> ProjectIdRequest<'static, R> {
        ProjectIdRequest {
            inner: self.inner.into_static(),
        }
    }
}

impl<'a, R: Resolver> Future for ProjectIdRequest<'a, R> {
    type Output = Result<ProjectId>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (_, bytes) = std::task::ready!(self.project().inner.poll(cx))?;
        let bytes = bytes.trim_ascii();

        if bytes.is_empty() {
            return Poll::Ready(Err(Error::invalid_data(
                "project id can't be an empty string",
            )));
        }

        Poll::Ready(Ok(ProjectId::from_byte_slice(bytes)?))
    }
}

pin_project_lite::pin_project! {
    pub struct LoadFuture<'a, R: Resolver = GaiResolver> {
        client: CowMut<'a, Option<HttpClient<R>>>,
        #[pin]
        project_id_request: TryMaybeDone<ProjectIdRequest<'static, R>>,
        #[pin]
        token_request: TryMaybeDone<RequestJson<'static, HttpConnector<R>, Token<Bearer>>>,
    }
}

pin_project_lite::pin_project! {
    pub struct TryLoadFuture<'a, R: Resolver = GaiResolver> {
        #[pin]
        inner: LoadFuture<'a, R>,
    }
}

impl<'a, R> Future for TryLoadFuture<'a, R>
where
    R: Resolver,
    LoadFuture<'a, R>: Future<Output = Result<LoadProviderResult<'static, MetadataServer>>>,
{
    type Output = Result<Option<LoadProviderResult<'static, MetadataServer>>>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(match std::task::ready!(self.project().inner.poll(cx)) {
            Ok(load_res) => Ok(Some(load_res)),
            Err(Error::NoProviderFound) => Ok(None),
            Err(err) if err.is_connect_error() => Ok(None),
            Err(err) => Err(err),
        })
    }
}
impl<'a> LoadFuture<'a> {
    pub(crate) fn new(mut client: CowMut<'a, Option<HttpClient>>) -> Self {
        let fut =
            ProjectIdRequest::new(client.get_or_insert_with(crate::client::HttpClient::new_http))
                .into_static();

        Self {
            client,
            project_id_request: TryMaybeDone::Future(fut),
            // use Gone as a placeholder value
            token_request: TryMaybeDone::Gone,
        }
    }
    pub fn start_token_request(mut self) -> Self {
        if matches!(self.token_request, TryMaybeDone::Gone) {
            let client = self.client.get_or_insert_with(HttpClient::new_http);
            let request = client.request_json(make_request(&TOKEN_URI)).into_static();
            self.token_request = TryMaybeDone::Future(request);
        }

        self
    }
}

impl<'a, R: Resolver> LoadFuture<'a, R> {
    pub(crate) fn into_static(self) -> LoadFuture<'static, R> {
        LoadFuture {
            client: self.client.take_into_static(),
            project_id_request: match self.project_id_request {
                TryMaybeDone::Future(fut) => TryMaybeDone::Future(fut.into_static()),
                TryMaybeDone::Done(bytes) => TryMaybeDone::Done(bytes),
                TryMaybeDone::Gone => TryMaybeDone::Gone,
            },
            token_request: match self.token_request {
                TryMaybeDone::Future(fut) => TryMaybeDone::Future(fut.into_static()),
                TryMaybeDone::Done(token) => TryMaybeDone::Done(token),
                TryMaybeDone::Gone => TryMaybeDone::Gone,
            },
        }
    }

    pub(crate) fn pin_take_http(self: &mut Pin<&mut Self>) -> Option<HttpClient<R>> {
        self.as_mut().project().client.take()
    }

    pub fn try_load(self) -> TryLoadFuture<'a, R> {
        TryLoadFuture { inner: self }
    }
}

impl<R: Resolver> TryLoadFuture<'_, R> {
    pub(crate) fn into_static(self) -> TryLoadFuture<'static, R> {
        TryLoadFuture {
            inner: self.inner.into_static(),
        }
    }

    pub(crate) fn pin_take_http(self: &mut Pin<&mut Self>) -> Option<HttpClient<R>> {
        self.as_mut().project().inner.pin_take_http()
    }
}

impl<'a> Future for LoadFuture<'a> {
    type Output = Result<LoadProviderResult<'static, MetadataServer>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let project_id_request = this.project_id_request.as_mut().get_mut();
        let token_request = this.token_request.get_mut();

        // poll one or both futures
        match (&mut *project_id_request, &mut *token_request) {
            (project_id_fut @ TryMaybeDone::Future(_), request_fut @ TryMaybeDone::Future(_)) => {
                if let Poll::Ready(error) = Pin::new(project_id_fut).poll(cx) {
                    error?;
                }

                if let Poll::Ready(error) = Pin::new(request_fut).poll(cx) {
                    error?;
                }
            }
            (project_id_fut @ TryMaybeDone::Future(_), _) => {
                std::task::ready!(Pin::new(project_id_fut).poll(cx))?;
            }
            (_, _) => (),
        }

        match project_id_request {
            TryMaybeDone::Gone => panic!("LoadFuture polled after completion"),
            TryMaybeDone::Future(_) => Poll::Pending,
            TryMaybeDone::Done(_) => {
                let project_id = this
                    .project_id_request
                    .take_output()
                    .expect("we just checked that this is done");

                let token_future = std::mem::replace(token_request, TryMaybeDone::Gone);

                let provider = MetadataServer {
                    client: this.client.take().unwrap_or_else(HttpClient::new_http),
                };

                Poll::Ready(Ok(LoadProviderResult {
                    provider,
                    project_id,
                    token_future: GetTokenFuture::try_maybe_http(token_future),
                }))
            }
        }
    }
}
