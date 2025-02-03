use std::ops::{Deref, DerefMut};

use http::HeaderValue;
use http::header::AsHeaderName;
use http::response::Parts;

use crate::Error;

#[derive(Clone)]
pub struct Response<B>(pub http::Response<B>);

pub trait ResponseErrorHandler {
    type Output;
}

impl<B> Response<B> {
    pub fn error_for_status(self) -> Result<Self, Error> {
        if self.status().is_server_error() || self.status().is_client_error() {
            todo!()
        }

        Ok(self)
    }

    pub fn into_parts(self) -> (Parts, B) {
        self.0.into_parts()
    }

    pub fn header(&self, name: impl AsHeaderName) -> Option<&HeaderValue> {
        self.headers().get(name)
    }

    pub fn header_str(&self, name: impl AsHeaderName) -> Option<&str> {
        self.headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
    }

    pub fn split_body(self) -> (B, Response<()>) {
        let (parts, body) = self.into_parts();
        (body, Response(http::Response::from_parts(parts, ())))
    }

    pub fn split_parts(self) -> (Parts, Self) {
        let (parts, body) = self.into_parts();
        (parts, Self(http::Response::new(body)))
    }
}

impl<B> From<http::Response<B>> for Response<B> {
    #[inline]
    fn from(value: http::Response<B>) -> Self {
        Self(value)
    }
}

impl<B> Into<http::Response<B>> for Response<B> {
    #[inline]
    fn into(self) -> http::Response<B> {
        self.0
    }
}

impl<B> Deref for Response<B> {
    type Target = http::Response<B>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<B> DerefMut for Response<B> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
