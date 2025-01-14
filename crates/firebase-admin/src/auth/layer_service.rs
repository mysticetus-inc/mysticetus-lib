use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::response::IntoResponse;
use http::{HeaderValue, header};
use jsonwebtoken::TokenData;
use tower::Layer;

use super::{AuthError, Claims, ValidateIdTokenShared};

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

impl<S, ReqBody> tower::Service<http::Request<ReqBody>> for ValidateIdTokenService<S>
where
    S: tower::Service<http::Request<ReqBody>> + Send + Clone + 'static,
    ReqBody: Send + 'static,
    S::Response: axum::response::IntoResponse,
    S::Error: From<std::convert::Infallible> + Send + 'static,
    S::Future: Send,
{
    type Response = axum::response::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        let token_header = match extract_header(&mut req) {
            Ok(token) => token,
            Err(error) => return Box::pin(std::future::ready(Ok(error.into_response()))),
        };

        let mut this = self.clone();

        Box::pin(async move {
            let res = validate_token(&token_header, &this.shared).await;
            req.headers_mut()
                .insert(header::AUTHORIZATION, token_header);

            let token = match res {
                Ok(token) => token,
                Err(error) => return Ok(error.into_response()),
            };

            req.extensions_mut().insert(token);

            let resp = this.service.call(req).await?;
            Ok(resp.into_response())
        })
    }
}

fn extract_header<Body>(req: &mut http::Request<Body>) -> Result<HeaderValue, AuthError> {
    let header = req
        .headers_mut()
        .remove(header::AUTHORIZATION)
        .ok_or(AuthError::NoBearerToken)?;

    if header.as_bytes().starts_with(b"Bearer ") {
        Ok(header)
    } else {
        req.headers_mut().insert(header::AUTHORIZATION, header);
        Err(AuthError::NoBearerToken)
    }
}

async fn validate_token(
    header: &http::HeaderValue,
    shared: &ValidateIdTokenShared,
) -> crate::Result<TokenData<Claims>> {
    let token = header
        .to_str()
        .map_err(AuthError::InvalidToken)?
        .trim_start_matches("Bearer ");

    shared
        .key_cache
        .validate_token(token, &shared.validation)
        .await
}
