use std::task::{Context, Poll};

use crate::backoff::BackoffConfig;
use crate::http_svc::HttpResponse;
use crate::retry::classify::ClassifyResponse;

pub mod body;
pub mod classify;
pub mod future;

#[derive(Debug, Default, Clone)]
pub struct RetryLayer<Classify> {
    backoff_config: BackoffConfig,
    classify: Classify,
}

impl<Classify> RetryLayer<Classify> {
    pub fn new(classify: Classify) -> Self {
        Self {
            backoff_config: BackoffConfig::default(),
            classify,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Retry<Svc, Classify> {
    backoff_config: BackoffConfig,
    classify: Classify,
    svc: Svc,
}

impl<Svc, Classify: Clone> tower::Layer<Svc> for RetryLayer<Classify> {
    type Service = Retry<Svc, Classify>;

    #[inline]
    fn layer(&self, svc: Svc) -> Self::Service {
        Retry {
            classify: self.classify.clone(),
            backoff_config: self.backoff_config,
            svc,
        }
    }
}

impl<Body, Svc, Classify> tower::Service<http::Request<Body>> for Retry<Svc, Classify>
where
    Body: body::UnpinBody,
    Svc: tower::Service<http::Request<body::RetryableBody<Body>>> + Clone,
    Svc::Response: HttpResponse,
    Classify: ClassifyResponse<Svc::Response, Svc::Error>,
{
    type Error = Svc::Error;
    type Response = Svc::Response;
    type Future = future::RetryFuture<Body, Svc, Classify>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        future::RetryFuture::new(
            self.svc.clone(),
            req,
            self.classify.clone(),
            &self.backoff_config,
        )
    }
}
