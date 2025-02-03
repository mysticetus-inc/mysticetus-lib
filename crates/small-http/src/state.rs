use std::future::Future;
use std::pin::Pin;

use hyper::body::Incoming;
use tokio_util::sync::ReusableBoxFuture;

use crate::HttpVersion;

pub struct ClientParts<B, HttpVers: HttpVersion<B>> {
    send_request: Option<HttpVers::SendRequest>,
    request_future: ReusableBoxFuture<
        'static,
        (
            HttpVers::SendRequest,
            hyper::Result<http::Response<Incoming>>,
        ),
    >,
}

impl<B: 'static, HttpVers: HttpVersion<B>> ClientParts<B, HttpVers> {
    pub fn new(send_request: HttpVers::SendRequest) -> Self {
        Self {
            send_request: Some(send_request),
            request_future: ReusableBoxFuture::new(std::future::pending()),
        }
    }
}

pub struct RequestFuture<'a, B, HttpVers: HttpVersion<B>> {
    parts: &'a mut ClientParts<B, HttpVers>,
}

impl<'a, B, HttpVers: HttpVersion<B>> Future for RequestFuture<'a, B, HttpVers> {
    type Output = hyper::Result<http::Response<Incoming>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}
