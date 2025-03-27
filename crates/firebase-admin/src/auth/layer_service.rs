use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::{HeaderValue, Request, Response, header};
use jsonwebtoken::TokenData;
use tower::{Layer, Service};

use super::key_cache::{KEY_CACHE, KeyId};
use super::{Claims, ValidateIdTokenShared, ValidateTokenError};

#[derive(Debug, Clone)]
pub struct ValidateIdTokenLayer {
    shared: Arc<ValidateIdTokenShared>,
}

impl ValidateIdTokenLayer {
    pub fn new(project_id: &'static str) -> Self {
        Self::from_parts(project_id, reqwest::Client::new())
    }

    pub fn from_parts(project_id: &'static str, client: reqwest::Client) -> Self {
        Self {
            shared: Arc::new(ValidateIdTokenShared::new(project_id, client, true)),
        }
    }
}

impl<S> Layer<S> for ValidateIdTokenLayer {
    type Service = ValidateIdTokenService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ValidateIdTokenService {
            service,
            shared: Arc::clone(&self.shared),
        }
    }
}

#[derive(Clone)]
pub struct ValidateIdTokenService<S> {
    service: S,
    shared: Arc<ValidateIdTokenShared>,
}

impl<S, ReqBody, RespBody> Service<Request<ReqBody>> for ValidateIdTokenService<S>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>> + Clone,
    RespBody: From<Cow<'static, str>>,
{
    type Response = Response<RespBody>;
    type Error = S::Error;
    type Future = ValidateIdTokenFuture<ReqBody, RespBody, S>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        match ValidateIdTokenFuture::try_build(req, self) {
            Ok(fut) => fut,
            Err(error) => ValidateIdTokenFuture::error(error),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = ValidateIdTokenFutureProject]
    pub enum ValidateIdTokenFuture<ReqBody, RespBody, S: Service<Request<ReqBody>, Response = Response<RespBody>>> {
        Error { error: crate::Error, logged: bool },
        PendingVerification {
            request: Option<Request<ReqBody>>,
            #[pin]
            validate_future: Pin<Box<dyn Future<Output = (HeaderValue, crate::Result<TokenData<Claims>>)> + Send + 'static>>,
            service: S,
        },
        Calling { #[pin] fut: S::Future },
    }
}

impl<ReqBody, RespBody, S> ValidateIdTokenFuture<ReqBody, RespBody, S>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>>,
{
    fn error(error: impl Into<crate::Error>) -> Self {
        Self::Error {
            error: error.into(),
            logged: false,
        }
    }

    fn try_build(
        mut request: Request<ReqBody>,
        service: &mut ValidateIdTokenService<S>,
    ) -> crate::Result<Self>
    where
        S: Clone,
    {
        let ValidateIdTokenService { service, shared } = service;

        let token_header = super::validate::extract_header(request.headers_mut())?;

        match super::validate::try_authorize_from_header(&token_header, &shared.validation)? {
            Ok(decoded) => {
                reassemble_request(token_header, decoded, &mut request);
                Ok(Self::Calling {
                    fut: service.call(request),
                })
            }
            Err((_token_str, key_id)) => {
                // we need to replace the original service with the cloned service.
                // this is because the cloned service might not be ready
                // (via poll_ready), even if the original one was.
                //
                // this comes up with services that are actually just detached
                // handles (i.e an mpsc::Sender), where the real work is done
                // in a background thread/task.
                //
                // see the issue in tower describing in more detail:
                // https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
                let mut service_clone = service.clone();
                std::mem::swap(&mut service_clone, service);

                let shared = Arc::clone(shared);

                let validate_future = Box::pin(async move {
                    let result = decode_token(key_id, &token_header, &shared).await;
                    (token_header, result)
                });

                Ok(Self::PendingVerification {
                    request: Some(request),
                    validate_future,
                    service: service_clone,
                })
            }
        }
    }
}

async fn decode_token(
    key_id: KeyId,
    token_header: &HeaderValue,
    shared: &ValidateIdTokenShared,
) -> crate::Result<TokenData<Claims>> {
    let token = token_header
        .to_str()
        .map_err(ValidateTokenError::InvalidToken)?
        .trim_start_matches("Bearer ");

    let decoder = KEY_CACHE.get_decoder(key_id, &shared.client).await?;

    decoder.decode_token(token, &shared.validation)
}

fn reassemble_request<ReqBody>(
    token_header: HeaderValue,
    decoded_token: TokenData<Claims>,
    request: &mut Request<ReqBody>,
) {
    request
        .headers_mut()
        .insert(header::AUTHORIZATION, token_header);
    request.extensions_mut().insert(decoded_token);
}

fn make_error_response<B>(error: &crate::Error) -> Response<B>
where
    B: From<Cow<'static, str>>,
{
    let (status, body) = error.to_response_parts();
    let mut resp = Response::new(B::from(body));
    *resp.status_mut() = status;
    resp
}

impl<ReqBody, RespBody, S> Future for ValidateIdTokenFuture<ReqBody, RespBody, S>
where
    S: Service<Request<ReqBody>, Response = Response<RespBody>>,
    RespBody: From<Cow<'static, str>>,
{
    type Output = Result<Response<RespBody>, S::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project() {
                ValidateIdTokenFutureProject::Error { error, logged } => {
                    if !*logged {
                        tracing::warn!(message = "failed to validate id token", ?error);
                        *logged = true;
                    }
                    return Poll::Ready(Ok(make_error_response(error)));
                }
                ValidateIdTokenFutureProject::PendingVerification {
                    request,
                    validate_future,
                    service,
                } => {
                    let (token_header, decoded_token_result) =
                        std::task::ready!(validate_future.poll(cx));

                    let mut request = request.take().expect("invalid state");
                    request
                        .headers_mut()
                        .insert(header::AUTHORIZATION, token_header);

                    match decoded_token_result {
                        Ok(decoded_token) => {
                            request.extensions_mut().insert(decoded_token);
                            let fut = service.call(request);
                            self.set(ValidateIdTokenFuture::Calling { fut });
                        }
                        Err(error) => self.set(ValidateIdTokenFuture::error(error)),
                    }
                }
                ValidateIdTokenFutureProject::Calling { fut } => return fut.poll(cx),
            }
        }
    }
}
